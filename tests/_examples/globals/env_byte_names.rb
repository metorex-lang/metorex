key = "ENV_BYTE_NAMES_ÜBER"
ENV[key] = "bar"
p ENV.delete(key.b)
p ENV[key]
ENV[key] = "value"
saved = ENV.to_hash
ENV.replace(saved.merge(key.b => "binary"))
p ENV[key]
ENV.replace(saved.reject { |name, _| name == key }.merge(key.encode(Encoding::ISO_8859_1) => "latin1"))
p ENV[key]
p ENV[key.encode(Encoding::ISO_8859_1)]
p ENV.keys.select { |name| name.b.start_with?("ENV_BYTE_NAMES") }.map(&:bytes)
ENV.replace(saved.reject { |name, _| name == key })
p ENV.keys.select { |name| name.b.start_with?("ENV_BYTE_NAMES") }
begin
  ENV.replace(saved.merge("ENV_BYTE_NAMES_FIRST" => "1", Object.new => Object.new))
rescue TypeError => error
  p error.message
end
p ENV["ENV_BYTE_NAMES_FIRST"]
ENV.delete("ENV_BYTE_NAMES_FIRST")
