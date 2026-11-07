require "openssl"

key = ["2b7e151628aed2a6abf7158809cf4f3c"].pack("H*")
counter = ["f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff"].pack("H*")
plain = ["6bc1bee22e409f96e93d7e117393172aae2d8a571e03ac9c9eb76fac45af8e51"].pack("H*")

cipher = OpenSSL::Cipher::AES.new(128, :CTR)
p cipher.name, cipher.block_size, cipher.iv_len, cipher.key_len
cipher.encrypt
cipher.key = key
cipher.iv = counter
whole = cipher.update(plain) + cipher.final
p whole.unpack1("H*")

cipher.iv = counter
pieces = [plain.byteslice(0, 5), plain.byteslice(5, 20), plain.byteslice(25, 7)].map { |piece| cipher.update(piece) }
p pieces.map(&:bytesize), pieces.join == whole

decipher = OpenSSL::Cipher.new("aes-128-ctr")
decipher.decrypt
decipher.key = key
decipher.iv = counter
p decipher.update(whole) == plain

wrapping = OpenSSL::Cipher::AES256.new(:CTR)
wrapping.encrypt
wrapping.key = "k" * 32
wrapping.iv = "\xff".b * 16
p wrapping.update("a" * 40).unpack1("H*")
p OpenSSL::Cipher::AES128.new.name, OpenSSL::Cipher::AES.new("192-CBC").name
