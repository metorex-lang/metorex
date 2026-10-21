class Matcher
  def check(text) = text =~ /\AAUTH=/i

  def labeled(name)
    case name
    when "UID" then 1
    when /\ABODY\[/ni then 2
    end
  end

  def as_value(name)
    found = case name
            when /\ABINARY\[/ni then 3
            end
    found
  end

  def words(list) = list.flat_map { _1.split(/ /, -1) }

  def args(keys)
    keys in /\ACHARSET\b/i | Array[/\ACHARSET\z/i, *]
  end
end

%i[check labeled as_value words args].each do |name|
  found = []
  walk = ->(item) { item.is_a?(Array) ? item.each(&walk) : (found << item if item.is_a?(Regexp)) }
  walk.(RubyVM::InstructionSequence.of(Matcher.instance_method(name)).to_a)
  p [name, found]
end
