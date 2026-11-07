text = "Mikel Linds\xE4r <m@x.net>, \xE4d <a@x.net>".dup.force_encoding("ISO-8859-1")

def show(label, value)
  shown = case value
          when String then [value.encoding.to_s, value.bytes.select { |byte| byte > 127 }]
          when Array then value.map { |held| held.is_a?(String) ? held.bytes.select { |byte| byte > 127 } : held }
          else value
          end
  p [label, shown]
end

show :gsub_string, text.gsub("Mikel", "M")
show :gsub_block, text.gsub(/\w+/) { |word| word }
show :sub, text.sub(/Linds/, "L")
show :index_range, text[6, 6]
show :slice_regexp, text[/L\S+/]
show :scan, text.scan(/\S+/)
show :chars, text.chars.last(12)
show :partition, text.partition(", ")
show :rpartition, text.rpartition(" ")
show :lines, text.lines
show :strip, " #{text} ".strip
show :tr, text.tr("M", "N")
show :delete, text.delete("M")
show :squeeze, text.squeeze
show :ljust, text.ljust(50, ".")
show :plus, text + text
show :times, text * 2
show :upcase, text.upcase
show :downcase, text.downcase
show :capitalize, text.capitalize
show :swapcase, text.swapcase
show :reverse, text.reverse
show :each_line, text.each_line(",").to_a
show :match_capture, text.match(/(\S+r)/)[1]
show :pre_match, text.match(/</).pre_match
show :post_match, text.match(/,/).post_match
show :dollar, (text =~ /(L\S+)/ && $1)
show :format, format("%s!", text)
show :dup, text.dup
show :slice_bang, text.dup.tap { |held| held.slice!(0, 6) }
show :insert, text.dup.insert(0, "x")
show :concat, text.dup << " more"
show :unpack, text.unpack1("a*")
show :each_char, text.each_char.to_a.last(10)
show :byteslice, text.byteslice(6, 7)
show :center, text.center(60)
show :chomp, "#{text}\n".chomp
show :chop, text.chop
show :split_limit, text.split(/,\s*/, 2)
show :start_with, text.start_with?("Mikel")
show :to_s, text.to_s
show :to_str, text.to_str
show :encode, text.encode("UTF-8")
show :freeze_dup, text.dup.freeze
show :interpolated, "#{text}"
a = "\xC3\xA9".dup.force_encoding("ISO-8859-1")
p a.size, a.chars.size, a.bytes, a[0].bytes, a.reverse.bytes
b = "\xC3\xA9".dup.force_encoding("Windows-1252")
p b.size, b.reverse.bytes
