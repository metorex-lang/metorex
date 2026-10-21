require "openssl"
require "tmpdir"

context = OpenSSL::SSL::SSLContext.new
p context.setup, context.frozen?, context.setup

key = OpenSSL::PKey::RSA.new(1024)
Dir.mktmpdir do |directory|
  path = File.join(directory, "server.key")
  File.write(path, key.to_pem)
  File.open(path) { |file| p OpenSSL::PKey::RSA.new(file).n == key.n }
end

[Object.new, 1.5].each do |given|
  OpenSSL::PKey::RSA.new(given)
rescue TypeError => error
  p error.message
end
