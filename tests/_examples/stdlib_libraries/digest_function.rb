# `Digest(name)` answers the digest class of that name, from anywhere, and a
# name Digest has no class for is a LoadError naming the library.
require "digest"

module Checksums
  def self.of(text) = Digest(:SHA256).hexdigest(text)
end

p Digest(:MD5)
p Checksums.of("invoice 1042")[0, 16]
begin
  Digest(:Ledger)
rescue LoadError => error
  p error.message
end
p Object.private_method_defined?(:Digest)
