require "openssl"
versions = {
  SSL3: OpenSSL::SSL::SSL3_VERSION,
  TLS1: OpenSSL::SSL::TLS1_VERSION,
  TLS1_1: OpenSSL::SSL::TLS1_1_VERSION,
  TLS1_2: OpenSSL::SSL::TLS1_2_VERSION,
}
%i[min_version= max_version=].each do |setter|
  versions.each do |name, version|
    context = OpenSSL::SSL::SSLContext.new
    begin
      context.public_send(setter, version)
      puts "#{setter} #{name}: accepted"
    rescue OpenSSL::SSL::SSLError => e
      puts "#{setter} #{name}: #{e.message}"
    end
  end
end
context = OpenSSL::SSL::SSLContext.new
begin
  context.max_version = :SSL3
rescue OpenSSL::SSL::SSLError => e
  puts "by name: #{e.message}"
end
begin
  context.ssl_version = :SSLv3
rescue OpenSSL::SSL::SSLError, ArgumentError => e
  puts "ssl_version: #{e.class}: #{e.message}"
end
