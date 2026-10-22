# Trimming a string whose bytes UTF-8 does not spell: from the front up to a
# first character that is whole, and never from the back.
["\xFF", "a \xFF ", " \xFF", "\xFF a ", "a\xFF  ", "  b\xFF"].each do |text|
  broken = text.dup.force_encoding("UTF-8")
  results = %i[lstrip rstrip strip].map do |name|
    answered = broken.send(name)
    [name, answered.bytes, answered.encoding.name]
  rescue ArgumentError, EncodingError => error
    [name, error.class, error.message]
  end
  p results
end
held = "  b\xFF".dup.force_encoding("UTF-8")
p held.lstrip!, held.bytes
frozen = "  b\xFF".dup.force_encoding("UTF-8").freeze
begin
  frozen.lstrip!
rescue FrozenError => error
  p error.class
end
