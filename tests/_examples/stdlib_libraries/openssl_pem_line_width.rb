require "openssl"

key = OpenSSL::PKey::EC.generate("prime256v1")
name = OpenSSL::X509::Name.parse("/CN=localhost")

certificate = OpenSSL::X509::Certificate.new
certificate.version = 2
certificate.serial = 11
certificate.subject = name
certificate.issuer = name
certificate.public_key = key
certificate.not_before = Time.at(1_700_000_000)
certificate.not_after = Time.at(1_800_000_000)
certificate.sign(key, "SHA256")

revocations = OpenSSL::X509::CRL.new
revocations.version = 1
revocations.issuer = name
revocations.last_update = Time.at(1_700_000_000)
revocations.next_update = Time.at(1_800_000_000)
revocations.sign(key, "SHA256")

[certificate.to_pem, revocations.to_pem].each do |pem|
  lines = pem.lines.map(&:chomp)
  puts lines.first
  puts lines[1..-2].map(&:length).uniq.max
  puts lines.last
  puts OpenSSL::X509::Certificate.new(pem).serial if pem.include?("CERTIFICATE")
end
