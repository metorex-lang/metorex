require "openssl"

pem = <<~PEM
  -----BEGIN RSA PRIVATE KEY-----
  MIICXAIBAAKBgQC2J5rovejjxDTJOIgLf3T8kiVTZNKBSwCU+VWCFohMrLZpsgHC
  GZUFpKX3hP9BPKumuCk/4iqGUR5ilJzVOqFfUXomrS6eNPMO+a9vAUWdechM9jyY
  0d2jeBH8JTblkANhM5IPm17A56Emif6CwF3Es7ynzp84SKFB2Fg3VV0n/wIDAQAB
  AoGAae9KlgSsRpo7ILMx2436mVDdSEygZ/UYl4sSAZ/dEMp5I6sUfq+mDSweIX9F
  e8kvAryjsLV1ibvvceF//Dom0m5luYBIBHFruAQc8iFm4taZPHbNm4n2CQGTlrBV
  QsQWqpqtRY8v0jpzxCOonyQb4TzDpcJn2Lo8q9lzc2wGvhECQQDqjbIeYG6JCHTO
  cyb35KqncqhKXZs4TfdCVNRDsPFTkIJytCn7pSsijnQaCRHxIdEKbwbpdKN68gJi
  ez4Ha/hTAkEAxs9i2kOTYL369SH9mZRKD7zwV2NVgsSDXJIgesxrDww1Imu9DVGT
  A55wx2DLfk372fP4d5DlTyVoSJxd6RwsJQJAQZWPSYkjPzPbHWL7cXNB+IJJ5klY
  Pp0mVyjLFYZjZD9TCmmwF4oujoIQmrsQhkEzZ1x2LU8M6zLCUfEt/ftFuwJANNnA
  v6EjNp6KbkwA7KjdH13szm5btw2iliNKexqIg1GNdn8JnNbIrDecZ4Q11HfKseAH
  d1GPhAC/vj3DQUr5+QJBALdPEkxtYPN47TN2t30ffgNfR9k2AOYbE0l5CniTaY4J
  cKpGw4oEuuYaasxgdFNUeWWyViS6VjsZhQ82AKb4eBc=
  -----END RSA PRIVATE KEY-----
PEM
by_digest = "B2DSUNXE2kohi/BgjhhSt1g4tOT6d4a1CwmyXhECbHiUUAt8PPZ04kNeNk+iFCweRQR79r6g9fDGvNCng2Gf9x3XjsV46Fhme2mDQAVhcPd6A0Us6J5TRsXnbiNYKhWN+YNtAnd7yKtZazsmPuGenDhS5dKA0X2J/O5sn1sec78=".unpack1("m")
by_max = "Vq2/7B4Mm4R2tNT6BEBPgycgyxM0Rgd7oolt+EtaD3ercgADxO/Rbm45XbCowDrtWoF4KBRNEnlNpmB7cqPyNI7USXUONJBLqC+dXbHuA6ZfyGCRGE7aa9KZG0SaQ6P1Oc8UawOnHj8jHnpf6xsoL+nhSKjrunMkUTPMej/jJvw=".unpack1("m")

key = OpenSSL::PKey::RSA.new(pem)
public_key = OpenSSL::PKey::RSA.new(key.public_to_pem)
p public_key.verify_pss("SHA256", by_digest, "quarterly report", salt_length: :digest, mgf1_hash: "SHA256")
p public_key.verify_pss("SHA256", by_digest, "quarterly report", salt_length: :auto, mgf1_hash: "SHA256")
p public_key.verify_pss("SHA256", by_digest, "annual report", salt_length: :digest, mgf1_hash: "SHA256")
begin
  public_key.verify_pss("SHA256", by_digest, "quarterly report", salt_length: :max, mgf1_hash: "SHA256")
rescue TypeError => error
  p error.message
end
p public_key.verify_pss("SHA384", by_max, "quarterly report", salt_length: 78, mgf1_hash: "SHA1")
p public_key.verify_pss("SHA384", by_max, "quarterly report", salt_length: :auto, mgf1_hash: "SHA256")

made = key.sign_pss("SHA512", "quarterly report", salt_length: 20, mgf1_hash: "SHA512")
p made.bytesize, made == key.sign_pss("SHA512", "quarterly report", salt_length: 20, mgf1_hash: "SHA512")
p public_key.verify_pss("SHA512", made, "quarterly report", salt_length: 20, mgf1_hash: "SHA512")
p public_key.verify_pss(OpenSSL::Digest.new("SHA512"), made, "quarterly report", salt_length: :auto, mgf1_hash: "SHA512")
begin
  public_key.sign_pss("SHA256", "quarterly report", salt_length: :digest, mgf1_hash: "SHA256")
rescue ArgumentError => error
  p error.message
end
