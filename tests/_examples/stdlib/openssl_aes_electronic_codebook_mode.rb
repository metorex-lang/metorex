require "openssl"

cipher = OpenSSL::Cipher.new("aes-256-ecb")
p cipher.iv_len, cipher.block_size, cipher.name
cipher.encrypt
cipher.key = "k" * 32
sealed = cipher.update("x" * 20) + cipher.final
p sealed.unpack1("H*")
p (cipher.update("x" * 16) + cipher.update("x" * 16)).unpack1("H*")
opened = OpenSSL::Cipher.new("aes-256-ecb").decrypt
opened.key = "k" * 32
p opened.update(sealed) + opened.final
p OpenSSL::Cipher::AES256.new(:ECB).name
p OpenSSL::Cipher.ciphers.include?("aes-128-ecb")
