value = "text/plain; name=ci\xC3\xABle.txt".b

spaced = value.gsub(/\s*=\s*/, "=").gsub(/[; ]+/, "; ").gsub(/;\s*$/, "")
p spaced.bytes.size

value =~ /((boundary|name|filename)=(\S*))/i
joined = "#{$`.downcase}#{$2}=#{$3}#{$'.downcase}"
p $3.bytes.size, joined.bytes.size, joined.encoding

joined =~ /([\w\-]+\/[\w\-]+);\s+(.*)/i
params = $2.to_s.split(/\s+/)
p params.map { |param| param.bytes.size }
params = params.map { |param| param.to_s.chomp.strip }
p params.map { |param| param.bytes.size }
params = params.map { |param| param.split(/\s*\=\s*/, 2) }
p params.map { |pair| pair.map { |part| part.bytes.size } }
quoted = params.map { |pair| "#{pair[0]}=\"#{pair[1].to_s.gsub(/;$/, "")}\"" }.join("; ")
p quoted.bytes.size, quoted.encoding
p "#{$1}; #{quoted}".bytes.size

piece = "name=ci\xC3\xABle.txt".b.split(/\s+/).first
p piece.chomp.bytes.size, piece.strip.bytes.size, piece.lstrip.bytes.size, piece.rstrip.bytes.size
plain = "ci\xC3\xAB ".b
p plain.chomp.bytes, plain.strip.bytes, plain.rstrip.bytes, plain.chomp(" ").bytes
