require "securerandom"

p defined?(Base64)
p SecureRandom.base64(12).length
p SecureRandom.urlsafe_base64(12).match?(/\A[A-Za-z0-9_-]+\z/)
require "base64"
p defined?(Base64)
