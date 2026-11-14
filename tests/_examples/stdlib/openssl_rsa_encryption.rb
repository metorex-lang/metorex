require "openssl"

KEY = OpenSSL::PKey::RSA.new(<<~PEM)
-----BEGIN RSA PRIVATE KEY-----
MIICXAIBAAKBgQCmScC8F96NImqYvgYXCEQKtK8iP+YlWX4DwudS8umRwV2G+J5f
Y0y7BcAq5Y629tIPAf/HVEeinURkaViGFbol1+s/X4b92b1GHa8viBUUdc4Bd22r
K/XHDlwt8ZsJ476lt0TtcsNdcugYbHon1b7Nu3ytNmicjQBWcKQpPoYK+wIDAQAB
AoGABOg8RdC2gpe3aZaB+4AE8gPu/yW/lu6TJ21DAlGkzuZT6j2//rSheQtsZler
5H5YKJU4hRtZmdTmRchOEnblrVgHvJiW/IiDFa3NDp8ShE9zho3M7dkptLOtEk2x
JY85w5VQ5+Wj3+J+cCGj2FckjO8G0wwmEUVZ6VaiFT7GuHECQQDXEgpx0qvf8fev
8z2cwcd4RrafLJoRjqg6p/LUSh+BnWnRt1QoabI9TLMdAeAxedXdhNZNQM2sP5ZQ
pksX11DHAkEAxe8Y0sGsUTUs9GMz0kod3xsDuEodRNHTuUKpaJLvGWq5PR24MTsF
s1BkgYIT4yihnnajbFEvDd3HuvBh4ERoLQJBAIFAbFp/e5+4sVxi2JJOuuwDw1jh
2y3WCDBIIt2GNm0VBzVr8f3fl6jTYYl7SRsy116rqIcA/RoZlOKfLd4EHp0CQFEx
EcqqHUg2etlItCInuvcgurbm1/i55ZKR8LHvy9puN8wFG+A96Mm0SXYCaKv+p8U7
vy+YYafbeKmLH2sCRIUCQBztXrzVR3pDCjAA3q0u2Klj+87S13VVgrQN6UBoo7k8
qKCn/TS6PaId47nMfSyKCcm1EU9V5Y9OSRW35l7YMlg=
-----END RSA PRIVATE KEY-----
PEM
# Written by MRI with OAEP padding for KEY, and with PKCS #1 v1.5 padding
# for another key, which KEY answers with a message drawn from itself and
# the ciphertext in place of an error.
OAEP = ["6257f33d98b1b7f4ee105e2242dcfe5e4624f788229f7b3ab760783920f847dfefec84cb32acb27e14884856563e4c64abed4c7a0b3b39f3a61853f73d53cc2a3eb7bf54df67cff4a82c583b525773933d58137ee567570b474bc5b60a11e235ee9648377fec7037e974d7cf1151782ee019830b70e0f70a81b6126780744038"].pack("H*")
OTHER = ["0b0449768a93521b041881ecbd4e366fe7060edb908431b86058646c0fb75dc949fea5f83cabd36e0f6901ccdc9cbd8f2f3dd6b7ca8a81b1b5a659e54646235213f7a4bbb9d5b4e7740f0cec10d4fdd855ac34994a44cdef6801a67aa9f7fd73a3dc1d243781693940ce564f4008d71dbe6ebea6618d6cba8ac0dd3bb2df9ad3"].pack("H*")

p KEY.private_decrypt(OAEP, OpenSSL::PKey::RSA::PKCS1_OAEP_PADDING)
p KEY.private_decrypt(OTHER).unpack1("H*")
p KEY.private_decrypt(KEY.public_encrypt("secret"))
p KEY.private_decrypt(KEY.public_encrypt("secret", OpenSSL::PKey::RSA::PKCS1_OAEP_PADDING), OpenSSL::PKey::RSA::PKCS1_OAEP_PADDING)
p KEY.public_decrypt(KEY.private_encrypt("signed"))
p KEY.public_key.public_encrypt("x" * 117).bytesize
[
  -> { KEY.public_encrypt("x" * 118) },
  -> { KEY.public_encrypt("x" * 87, OpenSSL::PKey::RSA::PKCS1_OAEP_PADDING) },
  -> { KEY.public_encrypt("x", OpenSSL::PKey::RSA::NO_PADDING) },
  -> { KEY.private_decrypt("x" * 129) },
  -> { KEY.public_key.private_decrypt(OAEP) },
  -> { KEY.private_decrypt(OTHER, OpenSSL::PKey::RSA::PKCS1_OAEP_PADDING) }
].each do |attempt|
  attempt.call
rescue OpenSSL::PKey::PKeyError => error
  p error.message
end
