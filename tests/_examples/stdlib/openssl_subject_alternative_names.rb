require "openssl"
ef = OpenSSL::X509::ExtensionFactory.new
ext = ef.create_extension("subjectAltName", "DNS:localhost")
p ext.oid, ext.value, ext.critical?
p ef.create_extension("subjectAltName", "DNS:example.com,IP:127.0.0.1").value
p ef.create_extension("subjectAltName", "email:a@example.com, URI:https://example.com/, IP:::1").value
p ef.create_extension("issuerAltName", "DNS:ca.example.com").to_der.unpack1("H*")
begin
  ef.create_extension("subjectAltName", "bogus:x")
rescue OpenSSL::X509::ExtensionError => error
  p error.class
end
