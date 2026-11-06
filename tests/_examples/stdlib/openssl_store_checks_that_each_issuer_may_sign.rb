require "openssl"
def certificate(subject, issuer, key, signer, version, extensions = [])
  made = OpenSSL::X509::Certificate.new
  made.version = version
  made.serial = 1
  made.subject = OpenSSL::X509::Name.parse(subject)
  made.issuer = issuer || made.subject
  made.public_key = key
  made.not_before = Time.now - 60
  made.not_after = Time.now + 3600
  factory = OpenSSL::X509::ExtensionFactory.new
  extensions.each { |name, value| made.add_extension(factory.create_extension(name, value, true)) }
  made.sign(signer, "SHA256")
end
root_key = OpenSSL::PKey::RSA.new(2048)
leaf_key = OpenSSL::PKey::RSA.new(2048)
{
  "v3 root without basic constraints" => [2, []],
  "v1 root" => [0, []],
  "v3 root with key usage only" => [2, [["keyUsage", "keyCertSign"]]],
  "v3 root that is no CA" => [2, [["basicConstraints", "CA:FALSE"]]],
  "v3 root that is a CA" => [2, [["basicConstraints", "CA:TRUE"]]]
}.each do |label, (version, extensions)|
  root = certificate("/CN=r", nil, root_key, root_key, version, extensions)
  leaf = certificate("/CN=l", root.subject, leaf_key, root_key, 2)
  forged = certificate("/CN=f", root.subject, root_key, leaf_key, 2)
  store = OpenSSL::X509::Store.new
  store.add_cert(root)
  calls = []
  store.verify_callback = proc { |ok, context| calls << [ok, context.error_depth, context.error]; ok }
  p [label, store.verify(leaf), store.error, calls]
  calls = []
  p [label, :forged, store.verify(forged), store.error, calls]
end
