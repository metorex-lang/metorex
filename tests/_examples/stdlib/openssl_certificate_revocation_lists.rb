require "openssl"

def certificate(subject, key, serial, issuer: nil, issuer_key: key, not_after: Time.now + 3600, authority: false, config: nil)
  made = OpenSSL::X509::Certificate.new
  made.version = 2
  made.serial = serial
  made.subject = subject
  made.issuer = issuer ? issuer.subject : subject
  made.public_key = key
  made.not_before = Time.now - 3600
  made.not_after = not_after
  factory = OpenSSL::X509::ExtensionFactory.new
  made.add_extension(factory.create_extension("basicConstraints", "CA:TRUE", true)) if authority
  if config
    factory.config = OpenSSL::Config.parse(config)
    factory.subject_certificate = made
    made.add_extension(factory.create_extension("crlDistributionPoints", "@points"))
  end
  made.sign(issuer_key, "sha256")
end

def revocation_list(issuer, key, revoked: [], next_update: Time.now + 3600)
  list = OpenSSL::X509::CRL.new
  list.issuer = issuer.subject
  list.version = 1
  list.last_update = Time.now - 3600
  list.next_update = next_update
  revoked.each do |serial|
    entry = OpenSSL::X509::Revoked.new
    entry.serial = serial
    entry.time = Time.now - 60
    entry.add_extension(OpenSSL::X509::Extension.new("CRLReason", OpenSSL::ASN1::Enumerated(1)))
    list.add_revoked(entry)
  end
  list.add_extension(OpenSSL::X509::Extension.new("crlNumber", OpenSSL::ASN1::Integer(1)))
  list.sign(key, "sha256")
end

def checked(root, leaf, lists)
  store = OpenSSL::X509::Store.new
  store.purpose = OpenSSL::X509::PURPOSE_ANY
  store.flags = OpenSSL::X509::V_FLAG_CRL_CHECK | OpenSSL::X509::V_FLAG_CRL_CHECK_ALL
  store.add_cert(root)
  lists.each { |list| store.add_crl(list) }
  context = OpenSSL::X509::StoreContext.new(store, leaf, [])
  [context.verify, context.error, context.error_string, context.current_cert&.subject&.to_s]
end

root_key = OpenSSL::PKey::RSA.new(1024)
leaf_key = OpenSSL::PKey::RSA.new(1024)
other_key = OpenSSL::PKey::RSA.new(1024)
root = certificate(OpenSSL::X509::Name.parse("/DC=org/DC=fake-ca/CN=Fake CA"), root_key, 1, authority: true)
points = "[points]\nURI.1 = http://www.example.com/crl\nURI.2 = http://backup.example.com/crl\n"
leaf = certificate(OpenSSL::X509::Name.parse("/DC=org/DC=fake/CN=Fake"), leaf_key, 2, issuer: root, issuer_key: root_key, config: points)
expired_leaf = certificate(OpenSSL::X509::Name.parse("/DC=org/DC=fake/CN=Fake"), leaf_key, 3, issuer: root, issuer_key: root_key, not_after: Time.now - 60)

p leaf.extensions.map(&:to_s)
list = revocation_list(root, root_key)
p list.version, list.issuer.to_s, list.signature_algorithm, list.extensions.map(&:to_s)
p list.verify(root_key), list.verify(other_key)
p list.to_pem.lines.first, OpenSSL::Config.parse(points)["points"]

p checked(root, leaf, [list])
p checked(root, expired_leaf, [list])
p checked(root, leaf, [])
p checked(root, leaf, [revocation_list(root, root_key, revoked: [2])])
p checked(root, leaf, [revocation_list(root, root_key, next_update: Time.now - 60)])
p checked(root, leaf, [revocation_list(root, other_key)])
revoked = revocation_list(root, root_key, revoked: [2]).revoked.first
p revoked.serial.to_i, revoked.extensions.map(&:to_s)
