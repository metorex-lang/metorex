# Writes src/regexp/unicode_properties.rs from the Onigmo property tables in
# the MRI release metorex follows: the code point ranges of every character
# property a pattern can name with \p{...} or [[:...:]], and the names that
# reach each one, as Onigmo reads them.
#
#   ruby scripts/generate_unicode_properties.rb

root = File.expand_path("..", __dir__)
release = File.join(root, "target", "ruby-release")
header = Dir.glob(File.join(release, "ruby-*", "enc", "unicode", "*", "name2ctype.h")).max
abort "no name2ctype.h under #{release}; unpack the release first" if header.nil?

# The header is written for the C preprocessor. MRI builds it with both
# property switches on, so the branches those keep are the ones read here.
defined = %w[USE_UNICODE_PROPERTIES USE_UNICODE_AGE_PROPERTIES]
kept_lines = []
taking = [true]
File.foreach(header) do |line|
  case line
  when /\A#\s*ifdef\s+(\w+)/
    taking.push(taking.last && defined.include?(Regexp.last_match(1)))
  when /\A#\s*ifndef\s+(\w+)/
    taking.push(taking.last && !defined.include?(Regexp.last_match(1)))
  when /\A#\s*if\b/
    # The one plain `#if` checks the character set and holds only an error.
    taking.push(false)
  when /\A#\s*else\b/
    outer = taking[-2]
    taking[-1] = outer && !taking[-1]
  when /\A#\s*endif\b/
    taking.pop
  else
    kept_lines << line if taking.last
  end
end
text = kept_lines.join

ranges_by_name = {}
text.scan(/static const OnigCodePoint CR_(\w+)\[\] = \{\s*(\d+),(.*?)\};/m) do |name, count, body|
  numbers = body.scan(/0x\h+/).map(&:hex)
  pairs = numbers.each_slice(2).to_a
  raise "#{name}: #{pairs.size} ranges where #{count} are counted" unless pairs.size == count.to_i
  ranges_by_name[name] = pairs
end

# Some properties share another's table, which a `#define` says.
text.scan(/^#define CR_(\w+) CR_(\w+)$/) do |alias_name, original|
  ranges_by_name[alias_name] = ranges_by_name.fetch(original)
end

listed = text[/CodeRanges\[\] = \{(.*?)\};/m, 1] or abort "no CodeRanges list"
order = listed.scan(/CR_(\w+)/).flatten
missing = order.reject { |name| ranges_by_name.key?(name) }
abort "CodeRanges names tables never defined: #{missing.join(", ")}" unless missing.empty?

pool = text.scan(/uniname2ctype_pool_str(\d+)\[sizeof\("([^"]+)"\)\]/).to_h
names = text.scan(/uniname2ctype_offset\(str(\d+)\),\s*(\d+)\}/).map do |slot, table|
  [pool.fetch(slot), Integer(table)]
end
names.sort_by!(&:first)
abort "no property names read" if names.empty?

# A table two names share is written once, and both point at it.
flat = []
placed = {}.compare_by_identity
tables = order.map do |name|
  ranges = ranges_by_name.fetch(name)
  placed[ranges] ||= begin
    first = flat.size
    flat.concat(ranges)
    [first, ranges.size]
  end
end

version = File.basename(File.dirname(header))
lines = []
lines << "// The character properties of Unicode #{version} as Onigmo names them, generated"
lines << "// by scripts/generate_unicode_properties.rb from MRI's name2ctype.h. Do not edit."
lines << ""
lines << "/// Every property's code point ranges, first and last, one property after"
lines << "/// another."
lines << "pub(crate) const PROPERTY_RANGES: &[(u32, u32)] = &["
flat.each_slice(4) do |row|
  lines << "    #{row.map { |low, high| format("(0x%x, 0x%x)", low, high) }.join(", ")},"
end
lines << "];"
lines << ""
lines << "/// Where each property's ranges start in PROPERTY_RANGES and how many it has,"
lines << "/// in the order Onigmo numbers the properties."
lines << "pub(crate) const PROPERTY_TABLES: &[(u32, u32)] = &["
tables.each_slice(6) do |row|
  lines << "    #{row.map { |first, count| "(#{first}, #{count})" }.join(", ")},"
end
lines << "];"
lines << ""
lines << "/// The names a pattern reaches each property by, lowercased with spaces,"
lines << "/// underscores, and hyphens left out, in sorted order."
lines << "pub(crate) const PROPERTY_NAMES: &[(&str, u16)] = &["
names.each { |name, table| lines << "    (#{name.inspect}, #{table})," }
lines << "];"

File.write(File.join(root, "src", "regexp", "unicode_properties.rs"), lines.join("\n") + "\n")
puts "#{order.size} properties, #{names.size} names, #{flat.size} ranges"
