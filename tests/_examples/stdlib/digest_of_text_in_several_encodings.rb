# A digest given strings in different encodings digests their bytes, as
# sprockets feeds it UTF-8 sources and binary data in turn.
require "digest/sha1"
require "digest/sha2"
digest = Digest::SHA1.new
digest << "café"
digest << "\xFF\xFE".b
digest << "é"
p digest.hexdigest
p Digest::SHA256.new.update("é").update("\xC3".b).hexdigest
