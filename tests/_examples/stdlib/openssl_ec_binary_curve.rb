require "openssl"

pem = <<~PEM
  -----BEGIN EC PRIVATE KEY-----
  MIHuAgEBBEgAw0fAClp5s8OvltdSYTBU4FTyxH4wWJ67CGx88WRh3h3N46JF9MYs
  IlxxE7AVckyx7xSugcaUTRuBY+U1ptFcOPMmpkeZhKygBwYFK4EEACehgZUDgZIA
  BAZrvHdo7mWpOaLeCVffgNkM4Xb5woJt9kBhFHARERfjHZ90Nmt3QOnDhqK0yw2V
  dTpTBLkP8c/55DI49gTa9C+a50MWiojZuwAXxUMi1fyvBubTtqoscyWEqosIX471
  MAFuxnkmWAcuA9h/RWgrdNU9yXiiexvL0tgwrLYay0mOAx585SxpdhOlYMVmKCqD
  ew==
  -----END EC PRIVATE KEY-----
PEM
signed_elsewhere = "MIGTAkgDvy8CZbHaV/MyMi1UcHUQdS7qz7VprKfzyNU1SfwZtaSqhn5yP7WDIBgUOQUhJTPIXCLIC00mHjq+87owZJ0l0zJvgOB+g/ICR1digwI4OOT5mRgDYzdRAz+r4X2IjA89krUtdcnxSgqgp/b/+ZkaW7U/OO2+2BuX4XdW6cCO0AcwyF1sPZMvRsZes4SBJgh7".unpack1("m")
digest = OpenSSL::Digest.digest("SHA512", "shipment 88 received")

key = OpenSSL::PKey.read(pem)
group = key.group
p group.curve_name, group.degree, key.check_key
p group.generator.on_curve?, group.generator.mul(group.order).infinity?
p group.generator.mul(key.private_key) == key.public_key
p key.to_pem == pem

public_only = OpenSSL::PKey::EC.new(key.public_to_pem)
p public_only.dsa_verify_asn1(digest, signed_elsewhere)
p public_only.dsa_verify_asn1(OpenSSL::Digest.digest("SHA512", "shipment 89 received"), signed_elsewhere)
p public_only.dsa_verify_asn1(digest, key.dsa_sign_asn1(digest))

point = key.public_key
compressed = point.to_octet_string(:compressed)
p compressed.bytesize, OpenSSL::PKey::EC::Point.new(group, compressed) == point
made = OpenSSL::PKey::EC.generate("sect571r1")
p made.public_key.on_curve?, made.private_key.num_bits <= 570
p OpenSSL::PKey::EC.builtin_curves.assoc("sect571r1")
%w[prime256v1 secp256k1 secp384r1 secp521r1 sect571r1].each { |c| g = OpenSSL::PKey::EC::Group.new(c); p [c, g.order.to_s(16), g.generator.to_octet_string(:uncompressed).unpack1("H*"), g.degree, g.cofactor.to_i] }
