require "openssl"

pem = <<~PEM
  -----BEGIN EC PRIVATE KEY-----
  MIGkAgEBBDBRIO0jaL2/JSgZDLWyg0S+xmDjstSXe9iqg9DLkJ1BQ8u8GQQ1OAcV
  Mg9NCsIaEVWgBwYFK4EEACKhZANiAAStCu8DcPYVn9XRetyJQAfwn4yhJrexECRo
  WNuv06CT2/5iavNuw+/Hx6UyFEFMDFgd5nT08d0+JmrQyYDYaOMzXC6Te8gDzihC
  +tiHolNlblFfZhHRn3BTZPpkIvskpr0=
  -----END EC PRIVATE KEY-----
PEM
signed_elsewhere = "MGQCMA0KemkkdQpG3k1wuzZgITECPXB6itkuquQteTE0uLPMOkFtdMskhJN3LKFiXmpDQAIwD1BjgtxOZgulpMdMks3aq8CnWyBs9xC2jybYnCb6OP8jBIDRO9ArvmBv2kzBQF8A".unpack1("m")
digest = OpenSSL::Digest.digest("SHA384", "invoice 2041 approved")

key = OpenSSL::PKey.read(pem)
p key.class, key.group.curve_name, key.group.degree, key.private?, key.check_key
p key.public_key.to_octet_string(:compressed).unpack1("H*")
p key.to_pem == pem

public_only = OpenSSL::PKey.read(key.public_to_pem)
p public_only.private?, public_only.public_key == key.public_key
p public_only.dsa_verify_asn1(digest, signed_elsewhere)
p public_only.dsa_verify_asn1(OpenSSL::Digest.digest("SHA384", "invoice 2042 approved"), signed_elsewhere)
p public_only.dsa_verify_asn1(digest, key.dsa_sign_asn1(digest))
p public_only.verify("SHA384", key.sign("SHA384", "memo"), "memo")

from_pkcs8 = OpenSSL::PKey::EC.new(key.private_to_pem)
p from_pkcs8.private_key == key.private_key, from_pkcs8.public_key == key.public_key

%w[prime256v1 secp256k1 secp521r1 prime239v2].each do |curve|
  made = OpenSSL::PKey::EC.generate(curve)
  group = made.group
  p [curve, made.public_key.on_curve?, group.generator.mul(group.order).infinity?]
  point = made.public_key
  p OpenSSL::PKey::EC::Point.new(group, point.to_octet_string(:compressed)) == point,
    OpenSSL::PKey::EC::Point.new(group, point.to_bn) == point
end

empty = OpenSSL::PKey::EC.new("prime256v1")
p empty.private?, empty.public?, empty.group.order.to_s(16)
begin
  public_only.dsa_verify_asn1(digest, "not a signature")
rescue OpenSSL::PKey::PKeyError => error
  p error.class
end
