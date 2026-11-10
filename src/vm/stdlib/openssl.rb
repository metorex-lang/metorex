# The parts of OpenSSL that are arithmetic over bytes: the message digests,
# the keyed digest built on them, the key derivation built on that, and the
# comparison that takes the same time however two strings differ.

require 'digest'
require 'securerandom'

module OpenSSL
  OPENSSL_VERSION = "OpenSSL 3.0.0"
  OPENSSL_VERSION_NUMBER = 0x30000000
  OPENSSL_LIBRARY_VERSION = OPENSSL_VERSION
  VERSION = "3.2.0"

  class OpenSSLError < StandardError; end

  # A message digest named the way OpenSSL names one.
  class Digest < ::Digest::Class
    class DigestError < OpenSSL::OpenSSLError; end

    # What each name stands for: how wide the digest is and how wide the
    # block it is computed over is.
    SHAPES = {
      "MD5" => [16, 64],
      "SHA1" => [20, 64],
      "SHA224" => [28, 64],
      "SHA256" => [32, 64],
      "SHA384" => [48, 128],
      "SHA512" => [64, 128]
    }

    attr_reader :name

    def initialize(name, data = nil)
      # Another digest names the same algorithm, and nothing of the state it
      # has built up comes along with it.
      name = name.name if name.is_a? OpenSSL::Digest
      unless name.is_a?(::String) || name.respond_to?(:to_str)
        raise TypeError, "no implicit conversion of #{name.class} into String"
      end
      held = name.is_a?(::String) ? name : name.to_str
      @name = OpenSSL::Digest.canonical_name held
      unless SHAPES.key? @name
        raise OpenSSL::Digest::DigestError, "Unsupported digest algorithm (#{held}).: unsupported"
      end
      super(data)
    end

    def self.canonical_name(held)
      named = held.to_s.upcase.gsub("-", "")
      named == "SHA" ? "SHA1" : named
    end

    def self.digest(name, data)
      new(name).digest(data)
    end

    def self.hexdigest(name, data)
      new(name).hexdigest(data)
    end

    def self.base64digest(name, data)
      new(name).base64digest(data)
    end

    def algorithm
      @name
    end

    def digest_length
      SHAPES[@name][0]
    end

    def block_length
      SHAPES[@name][1]
    end

    # Each name is also a class of its own, so `OpenSSL::Digest::SHA1.new`
    # names the same algorithm `OpenSSL::Digest.new("SHA1")` does.
    SHAPES.each_key do |named|
      built = ::Class.new(OpenSSL::Digest) do
        define_method :initialize do |data = nil|
          super(named, data)
        end

        %i[digest hexdigest base64digest].each do |reading|
          define_singleton_method(reading) { |data| new.public_send(reading, data) }
        end
      end
      const_set named, built
    end
  end

  class HMACError < OpenSSLError; end

  # A digest keyed with a secret, which says both what the message is and
  # that whoever wrote it held the key.
class HMAC
  # A keyed digest of what is fed to it, read as often as wanted.
  def initialize(key, digest)
    @key = key.to_s
    @digest_name = digest.respond_to?(:name) ? digest.name : digest.to_s
    @data = "".b
  end

  def initialize_copy(other)
    super
    @data = other.instance_variable_get(:@data).dup
  end

  def update(text)
    @data << text.to_s.b
    self
  end
  alias << update

  def reset
    @data = "".b
    self
  end

  def digest
    HMAC.computed(@digest_name, @key, @data)
  end

  def hexdigest
    ::Digest.hexencode digest
  end
  alias to_s hexdigest
  alias inspect hexdigest

  def base64digest
    [digest].pack "m0"
  end

  def ==(other)
    other.is_a?(HMAC) && other.digest == digest
  end

  def self.digest(digest, key, data)
      OpenSSL::HMAC.computed digest, key, data
    end

    def self.hexdigest(digest, key, data)
      ::Digest.hexencode computed(digest, key, data)
    end

    def self.base64digest(digest, key, data)
      [computed(digest, key, data)].pack "m0"
    end

    # The keyed digest itself: the key is brought to one block, and the
    # message is digested once inside a padded block and once outside it.
    def self.computed(digest, key, data)
      named = digest.respond_to?(:name) ? digest.name : digest.to_s
      held = OpenSSL::Digest.new named
      block = held.block_length
      shortened = key.to_s
      shortened = OpenSSL::Digest.new(named).digest(shortened) if shortened.length > block
      shortened = shortened + ("\x00" * (block - shortened.length))
      inner = OpenSSL::HMAC.masked shortened, 0x36
      outer = OpenSSL::HMAC.masked shortened, 0x5c
      once = OpenSSL::Digest.new(named).digest(inner + data.to_s)
      OpenSSL::Digest.new(named).digest(outer + once)
    end

    # The key with every byte of it masked by the same value.
    def self.masked(key, mask)
      key.each_char.map { |held| (held.ord ^ mask).chr }.join
    end
  end

  # Turning a password into a key, which takes long enough to do that
  # guessing the password one try at a time is not worth the wait.
  module KDF
    class KDFError < OpenSSL::OpenSSLError; end

    def self.pbkdf2_hmac(pass, salt:, iterations:, length:, hash:)
      unless hash.is_a?(::String) || hash.is_a?(OpenSSL::Digest) || hash.respond_to?(:to_str)
        raise TypeError, "wrong argument type #{hash.class} (expected OpenSSL/Digest)"
      end
      held = OpenSSL::KDF.coerced pass, "pass"
      seasoning = OpenSSL::KDF.coerced salt, "salt"
      rounds = OpenSSL::KDF.counted iterations, "iterations"
      wanted = OpenSSL::KDF.counted length, "length"
      named = hash.respond_to?(:name) ? hash.name : hash.to_s
      raise KDFError, "PKCS5_PBKDF2_HMAC: invalid iteration count" if rounds < 1
      raise KDFError, "invalid length" if wanted < 0
      return "" if wanted == 0
      # The rounds number in the tens of thousands and each is a pair of
      # digests, so the interpreter carries them.
      ::Digest.__pbkdf2__ OpenSSL::Digest.canonical_name(named), held, seasoning, rounds, wanted
    end

    SCRYPT_KEYWORDS = %i[salt N r p length].freeze

    # scrypt takes every one of its parameters by keyword. N, the size of
    # the table each block is mixed through, is a power of two above 1, and
    # r and p are positive.
    def self.scrypt(pass, **options)
      missing = SCRYPT_KEYWORDS - options.keys
      unless missing.empty?
        named = missing.map(&:inspect).join(", ")
        raise ArgumentError, "missing keyword#{"s" if missing.size > 1}: #{named}"
      end
      unknown = options.keys - SCRYPT_KEYWORDS
      unless unknown.empty?
        named = unknown.map(&:inspect).join(", ")
        raise ArgumentError, "unknown keyword#{"s" if unknown.size > 1}: #{named}"
      end
      held = OpenSSL::KDF.coerced pass
      seasoning = OpenSSL::KDF.coerced options[:salt]
      table = OpenSSL::KDF.counted options[:N]
      width = OpenSSL::KDF.counted options[:r]
      blocks = OpenSSL::KDF.counted options[:p]
      wanted = OpenSSL::KDF.counted options[:length]
      raise ArgumentError, "negative string size (or size too big)" if wanted < 0
      if table < 2 || (table & (table - 1)) != 0 || width < 1 || blocks < 1
        raise KDFError, "EVP_PBE_scrypt"
      end
      ::Digest.__scrypt__ held, seasoning, table, width, blocks, wanted
    end

    def self.coerced(value, _what = nil)
      return value if value.is_a? ::String
      unless value.respond_to? :to_str
        named = value.nil? || value == true || value == false ? value.inspect : value.class
        raise TypeError, "no implicit conversion of #{named} into String"
      end
      value.to_str
    end

    def self.counted(value, _what = nil)
      return value if value.is_a? Integer
      unless value.respond_to? :to_int
        raise TypeError, "no implicit conversion of #{value.class} into Integer"
      end
      value.to_int
    end
  end

  # PBKDF2 under the name OpenSSL gave it before KDF, which Active Support's
  # key generator still calls.
  module PKCS5
    module_function

    def pbkdf2_hmac(pass, salt, iter, keylen, digest)
      OpenSSL::KDF.pbkdf2_hmac(pass, salt: salt, iterations: iter,
                               length: keylen, hash: digest)
    end

    def pbkdf2_hmac_sha1(pass, salt, iter, keylen)
      pbkdf2_hmac(pass, salt, iter, keylen, "sha1")
    end
  end

  # Bytes nobody can guess, which is what a key or a salt is made of.
  module Random
    def self.random_bytes(count = nil)
      SecureRandom.bytes OpenSSL::KDF.counted(count.nil? ? 16 : count, "length")
    end

    def self.pseudo_bytes(count = nil)
      random_bytes count
    end

    def self.seed(_text)
      nil
    end
  end

  # A cipher named the way OpenSSL names one. AES in CBC mode is carried,
  # with keys of 128, 192, and 256 bits, padded as PKCS#7 pads, and other
  # names are refused.
  class Cipher
    class CipherError < OpenSSLError; end

    BLOCK = 16
    KEY_BYTES = { "128" => 16, "192" => 24, "256" => 32 }.freeze

    def self.ciphers
      %w[aes-128-cbc aes-128-ctr aes-192-cbc aes-192-ctr aes-256-cbc aes-256-ctr]
    end

    attr_reader :name, :key_len

    def initialize name
      @name = name.to_s.upcase
      bits, mode = @name.match(/\AAES-(128|192|256)-(CBC|CTR)\z/)&.captures
      raise CipherError, "unsupported cipher algorithm (#{name})" if bits.nil?
      @counting = mode == "CTR"
      @key_len = KEY_BYTES[bits]
      @padding = 1
      @encrypting = true
      @iv = ("\0" * BLOCK).b
      @chain = @iv
      @pending = "".b
      @unused = "".b
    end

    def iv_len = BLOCK

    def key_len = @key_len

    # A counter turns the block cipher into a stream, which takes any length.
    def block_size = @counting ? 1 : BLOCK

    def encrypt
      @encrypting = true
      reset
    end

    def decrypt
      @encrypting = false
      reset
    end

    def key=(key)
      key = key.to_str
      raise ArgumentError, "key must be #{@key_len} bytes" unless key.bytesize == @key_len
      @key = key.b
    end

    def iv=(iv)
      iv = iv.to_str
      raise ArgumentError, "iv must be #{BLOCK} bytes" unless iv.bytesize == BLOCK
      @iv = iv.b
      @unused = "".b
      @chain = @iv
    end

    def random_key
      self.key = OpenSSL::Random.random_bytes(@key_len)
    end

    def random_iv
      self.iv = OpenSSL::Random.random_bytes(BLOCK)
    end

    def padding=(padding)
      @padding = padding
    end

    def update(data)
      data = data.to_str
      raise ArgumentError, "data must not be empty" if data.empty?
      raise CipherError, "key not set" if @key.nil?
      return counted(data.b) if @counting

      @pending << data.b
      whole = @pending.bytesize / BLOCK * BLOCK
      # Decrypting holds the last block back, since `final` takes its padding off.
      whole -= BLOCK if !@encrypting && @padding != 0 && whole == @pending.bytesize && whole.positive?
      run whole
    end

    def final
      raise CipherError, "key not set" if @key.nil?
      return "".b if @counting

      if @encrypting && @padding != 0
        count = BLOCK - @pending.bytesize % BLOCK
        @pending << (count.chr * count)
      end
      unless (@pending.bytesize % BLOCK).zero?
        raise CipherError, @encrypting ? "data not multiple of block length" : "wrong final block length"
      end
      answer = run @pending.bytesize
      answer = unpadded answer if !@encrypting && @padding != 0
      @chain = @iv
      answer
    end

    private

    def reset
      @chain = @iv
      @pending = "".b
      @unused = "".b
      self
    end

    # Runs `data` through the keystream, taking first what the last call left
    # of its final block.
    def counted(data)
      answer = "".b
      unless @unused.empty?
        taken = [@unused.bytesize, data.bytesize].min
        answer << data.byteslice(0, taken).bytes.zip(@unused.bytes).map { |byte, key| byte ^ key }.pack("C*")
        @unused = @unused.byteslice(taken, @unused.bytesize - taken)
        data = data.byteslice(taken, data.bytesize - taken)
      end
      return answer if data.empty?

      output, @chain, @unused = OpenSSL.__aes_ctr__(@key, @chain, data)
      answer << output
    end

    def run(length)
      return "".b if length.zero?
      taken = @pending.byteslice(0, length)
      @pending = @pending.byteslice(length, @pending.bytesize - length)
      answer, @chain = OpenSSL.__aes_cbc__(@key, @chain, taken, @encrypting)
      answer.b
    end

    def unpadded(text)
      count = text.getbyte(-1)
      if count.nil? || count.zero? || count > BLOCK || text.byteslice(-count, count).bytes.uniq != [count]
        raise CipherError, "bad decrypt"
      end
      text.byteslice(0, text.bytesize - count)
    end
  end

  # `Cipher::AES.new(128, :CTR)` names the cipher by its parts, and
  # `Cipher::AES128.new(:CTR)` by its mode alone.
  class Cipher
    class AES < Cipher
      def initialize(*parts)
        super(parts.inject("AES") { |named, part| "#{named}-#{part}" }.downcase)
      end
    end

    %w[128 192 256].each do |bits|
      const_set("AES#{bits}", Class.new(Cipher) do
        define_method(:initialize) { |mode = "CBC"| super("aes-#{bits}-#{mode}".downcase) }
      end)
    end
  end

  module SSL
    # How a connection checks the certificate the other side presents, with
    # the values OpenSSL gives them.
    VERIFY_NONE = 0
    VERIFY_PEER = 1
    VERIFY_FAIL_IF_NO_PEER_CERT = 2
    VERIFY_CLIENT_ONCE = 4

    class SSLError < OpenSSL::OpenSSLError; end

    class SSLErrorWaitReadable < SSLError
      include IO::WaitReadable
    end

    class SSLErrorWaitWritable < SSLError
      include IO::WaitWritable
    end
  end

  # Comparing two strings without letting how long it takes say where they
  # first differ.
  def self.secure_compare(left, right)
    OpenSSL.fixed_length_secure_compare(
      ::Digest::SHA256.digest(OpenSSL::KDF.coerced(left, "a")),
      ::Digest::SHA256.digest(OpenSSL::KDF.coerced(right, "b"))
    ) && left == right
  end

  def self.fixed_length_secure_compare(left, right)
    held = OpenSSL::KDF.coerced left, "a"
    other = OpenSSL::KDF.coerced right, "b"
    unless held.length == other.length
      raise ArgumentError, "inputs must be of equal length"
    end
    differing = 0
    held.each_char.each_with_index do |piece, at|
      differing = differing | (piece.ord ^ other[at].ord)
    end
    differing == 0
  end
end

module OpenSSL
  # The universal ASN.1 tags, by the names OpenSSL gives them.
  module ASN1
    UNIVERSAL_TAG_NAME = [
      "EOC", "BOOLEAN", "INTEGER", "BIT_STRING", "OCTET_STRING", "NULL", "OBJECT",
      "OBJECT_DESCRIPTOR", "EXTERNAL", "REAL", "ENUMERATED", "EMBEDDED_PDV", "UTF8STRING",
      "RELATIVE_OID", nil, nil, "SEQUENCE", "SET", "NUMERICSTRING", "PRINTABLESTRING",
      "T61STRING", "VIDEOTEXSTRING", "IA5STRING", "UTCTIME", "GENERALIZEDTIME", "GRAPHICSTRING",
      "ISO64STRING", "GENERALSTRING", "UNIVERSALSTRING", "CHARACTER_STRING", "BMPSTRING"
    ].freeze
    UNIVERSAL_TAG_NAME.each_with_index do |named, tag|
      const_set named, tag unless named.nil?
    end

    # The tag and content of the DER value at the start of `bytes`, and the
    # bytes after it, or nil when they do not hold one.
    def self.__read_der__(bytes)
      return nil if bytes.bytesize < 2
      tag = bytes.getbyte(0)
      length = bytes.getbyte(1)
      offset = 2
      if length >= 0x80
        count = length & 0x7f
        return nil if count.zero? || bytes.bytesize < 2 + count
        length = bytes.byteslice(2, count).unpack1("H*").to_i(16)
        offset += count
      end
      return nil if bytes.bytesize < offset + length
      [tag, bytes.byteslice(offset, length), bytes.byteslice(offset + length..)]
    end

    # Every DER value one after another in `bytes`.
    def self.__read_all__(bytes)
      found = []
      rest = bytes
      until rest.empty?
        held = __read_der__(rest)
        return nil if held.nil?
        found << held[0, 2]
        rest = held[2]
      end
      found
    end

    def self.__integer_value__(content)
      content.unpack1("H*").to_i(16)
    end

    # The dotted text of an OID from its DER content.
    def self.__oid_text__(content)
      arcs = []
      value = 0
      content.each_byte do |byte|
        value = (value << 7) | (byte & 0x7f)
        next if byte >= 0x80
        arcs << value
        value = 0
      end
      first = arcs.shift
      leading = first < 80 ? [first / 40, first % 40] : [2, first - 80]
      (leading + arcs).join(".")
    end

    # A UTCTime or GeneralizedTime as a Time in UTC.
    def self.__time_value__(tag, content)
      digits = content.delete_suffix("Z")
      if tag == UTCTIME
        year = digits[0, 2].to_i
        digits = "#{year < 50 ? 20 : 19}#{digits}"
      end
      Time.utc(digits[0, 4].to_i, digits[4, 2].to_i, digits[6, 2].to_i,
               digits[8, 2].to_i, digits[10, 2].to_i, digits[12, 2].to_i)
    end

    # DER: a tag, the length of the content, and the content.
    def self.__encoded__(tag, content)
      bytes = content.b
      size = bytes.bytesize
      length = if size < 0x80
                 size.chr
               else
                 digits = []
                 while size > 0
                   digits.unshift(size & 0xff)
                   size >>= 8
                 end
                 (0x80 | digits.size).chr + digits.pack("C*")
               end
      (tag.chr + length).b + bytes
    end
  end

  module X509
    class NameError < OpenSSL::OpenSSLError; end

    # A distinguished name: a list of attributes, each an object with a
    # value and the ASN.1 string type the value is written in.
    class Name
      include Comparable

      COMPAT = 0
      RFC2253 = 17_892_119
      ONELINE = 8_520_479
      MULTILINE = 44_302_342
      DEFAULT_OBJECT_TYPE = OpenSSL::ASN1::UTF8STRING
      OBJECT_TYPE_TEMPLATE = Hash.new(OpenSSL::ASN1::UTF8STRING).merge(
        "C" => OpenSSL::ASN1::PRINTABLESTRING,
        "countryName" => OpenSSL::ASN1::PRINTABLESTRING,
        "serialNumber" => OpenSSL::ASN1::PRINTABLESTRING,
        "dnQualifier" => OpenSSL::ASN1::PRINTABLESTRING,
        "DC" => OpenSSL::ASN1::IA5STRING,
        "domainComponent" => OpenSSL::ASN1::IA5STRING,
        "emailAddress" => OpenSSL::ASN1::IA5STRING
      ).freeze

      # The ASN.1 string types OpenSSL takes a name's value in. Two others
      # fail in its ASN.1 library, and the rest are refused as the entry's
      # value.
      ACCEPTED_VALUE_TYPES = [3, 7, 8, 9, 11, 12, 13, 14, 15, 18, 19, 20, 22, 29].freeze
      LIBRARY_REFUSED_TYPES = [1, 6].freeze

      # "/A=B/C=D" or "A=B, C=D".
      def self.parse_openssl(text, template = OBJECT_TYPE_TEMPLATE)
        pairs = if text.start_with?("/")
                  text[1..].split("/").map { |part| part.split("=", 2) }
                else
                  text.split(",").map { |part| part.strip.split("=", 2) }
                end
        new(pairs, template)
      end

      def self.parse(text, template = OBJECT_TYPE_TEMPLATE)
        parse_openssl(text, template)
      end

      def initialize(entries = nil, template = OBJECT_TYPE_TEMPLATE)
        @entries = []
        return if entries.nil?
        return __read__(entries) if entries.is_a?(String)
        entries.each do |oid, value, type|
          type ||= template[oid]
          add_entry(oid, value, type)
        end
      end

      # A copy holds entries of its own, so adding one to it leaves the
      # original alone.
      def initialize_copy(other)
        super
        @entries = other.instance_variable_get(:@entries).map(&:dup)
        @canonical_encoding = nil
      end

      def add_entry(oid, value, type = nil, loc: -1, set: 0)
        named = oid.to_s
        text = OpenSSL::KDF.coerced(value)
        found = OpenSSL.__object__(:find, named)
        if found.nil? || found[2].nil?
          raise NameError, "X509_NAME_add_entry_by_txt: invalid field name (name=#{named})"
        end
        type = OBJECT_TYPE_TEMPLATE[named] if type.nil?
        if LIBRARY_REFUSED_TYPES.include?(type)
          raise NameError, "X509_NAME_add_entry_by_txt: ASN1 lib"
        end
        unless ACCEPTED_VALUE_TYPES.include?(type)
          raise NameError, "X509_NAME_add_entry_by_txt: nested asn1 error (Field=value, Type=X509_NAME_ENTRY)"
        end
        stored = type == OpenSSL::ASN1::UTF8STRING ? text.dup.force_encoding("UTF-8") : text.b
        entry = { short: found[0], long: found[1], oid: found[2], value: stored, type: type }
        at = loc.negative? ? @entries.size : [loc, @entries.size].min
        @entries.insert(at, entry)
        @canonical_encoding = nil
        self
      end

      # The entries of a name written in DER: a SEQUENCE of SETs, each
      # holding the SEQUENCE of an object and its value.
      def __read__(der)
        top = OpenSSL::ASN1.__read_der__(der.b)
        raise NameError, "X509_NAME: nested asn1 error" unless top && top[0] == 0x30
        OpenSSL::ASN1.__read_all__(top[1]).each do |_, set|
          OpenSSL::ASN1.__read_all__(set).each do |_, pair|
            (_, oid_bytes), (type, value) = OpenSSL::ASN1.__read_all__(pair)
            oid = OpenSSL::ASN1.__oid_text__(oid_bytes)
            found = OpenSSL.__object__(:find, oid)
            stored = type == OpenSSL::ASN1::UTF8STRING ? value.dup.force_encoding("UTF-8") : value.b
            @entries << { short: found[0], long: found[1], oid: oid, value: stored, type: type }
          end
        end
      end
      private :__read__

      def to_a
        @entries.map { |entry| [entry[:short], entry[:value], entry[:type]] }
      end

      def to_s(format = nil)
        case format
        when nil then @entries.map { |entry| "/#{entry[:short]}=#{oneline_value(entry[:value])}" }.join
        when RFC2253 then @entries.reverse.map { |entry| "#{entry[:short]}=#{rfc2253_value(entry[:value], false)}" }.join(",")
        when ONELINE then @entries.map { |entry| "#{entry[:short]} = #{quoted_value(entry[:value])}" }.join(", ")
        when MULTILINE
          @entries.map { |entry| "#{entry[:long].ljust(25)} = #{control_escaped(entry[:value])}" }.join("\n")
        else @entries.map { |entry| "#{entry[:short]}=#{oneline_value(entry[:value])}" }.join(", ")
        end
      end

      def to_utf8
        @entries.reverse.map { |entry| "#{entry[:short]}=#{rfc2253_value(entry[:value], true)}" }.join(",")
          .force_encoding("UTF-8")
      end

      def inspect
        "#<#{self.class.name} #{to_utf8}>"
      end

      def pretty_print(printer)
        printer.text(inspect)
      end

      def to_der
        sets = @entries.map do |entry|
          pair = OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::OBJECT, OpenSSL.__object__(:encode, entry[:oid])) +
                 OpenSSL::ASN1.__encoded__(entry[:type], entry[:value])
          OpenSSL::ASN1.__encoded__(0x31, OpenSSL::ASN1.__encoded__(0x30, pair))
        end
        OpenSSL::ASN1.__encoded__(0x30, sets.join.b)
      end

      # The order OpenSSL compares names in: by the length of their canonical
      # encoding, then by its bytes. The canonical encoding writes each value
      # as UTF-8, lowercased, with spaces trimmed and runs of them collapsed.
      def cmp(other)
        return nil unless other.is_a?(Name)
        mine = canonical_encoding
        theirs = other.canonical_encoding
        held = mine.bytesize <=> theirs.bytesize
        held = mine <=> theirs if held.zero?
        held
      end

      def <=>(other)
        cmp(other)
      end

      def eql?(other)
        other.is_a?(Name) && cmp(other).zero?
      end

      def hash
        canonical_encoding.hash
      end

      alias hash_old hash

      # Kept until an entry is added, since every comparison of two names
      # reads it.
      def canonical_encoding
        @canonical_encoding ||= @entries.map do |entry|
          value = entry[:value].b.downcase.strip.gsub(/\s+/, " ")
          pair = OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::OBJECT, OpenSSL.__object__(:encode, entry[:oid])) +
                 OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::UTF8STRING, value)
          OpenSSL::ASN1.__encoded__(0x31, OpenSSL::ASN1.__encoded__(0x30, pair))
        end.join.b
      end
      protected :canonical_encoding

      private

      # X509_NAME_oneline: '/' and '+' escaped, and bytes outside printable
      # ASCII written as \xHH.
      def oneline_value(value)
        value.b.each_byte.map do |byte|
          if byte == 0x2f || byte == 0x2b then "\\#{byte.chr}"
          elsif byte < 0x20 || byte > 0x7e then format("\\x%02X", byte)
          else byte.chr
          end
        end.join
      end

      # RFC 2253: the separators and the quoting characters escaped, a
      # leading '#' or space and a trailing space escaped, and control bytes
      # written as \HH. Bytes above ASCII are \HH too, unless the name is
      # written as UTF-8.
      def rfc2253_value(value, utf8)
        bytes = value.b.bytes
        last = bytes.size - 1
        written = +""
        index = 0
        while index < bytes.size
          byte = bytes[index]
          if byte >= 0x80 && utf8
            width = byte >= 0xf0 ? 4 : byte >= 0xe0 ? 3 : byte >= 0xc0 ? 2 : 1
            written << bytes[index, width].pack("C*").force_encoding("UTF-8")
            index += width
            next
          end
          written << if ",+\"\\<>;".include?(byte.chr) then "\\#{byte.chr}"
                     elsif index.zero? && (byte == 0x23 || byte == 0x20) then "\\#{byte.chr}"
                     elsif index == last && byte == 0x20 then "\\ "
                     elsif byte < 0x20 || byte >= 0x7f then format("\\%02X", byte)
                     else byte.chr
                     end
          index += 1
        end
        written
      end

      # ONELINE: a value holding a separator, or with a leading '#' or space
      # or a trailing space, is quoted. Quotes and backslashes are escaped
      # either way.
      def quoted_value(value)
        bytes = value.b
        escaped = control_escaped(bytes.gsub(/["\\]/) { |held| "\\#{held}" })
        needs_quotes = bytes.match?(/[,+<>;]/) || bytes.start_with?("#", " ") || bytes.end_with?(" ")
        needs_quotes ? "\"#{escaped}\"" : escaped
      end

      def control_escaped(value)
        value.b.each_byte.map { |byte| byte < 0x20 || byte >= 0x7f ? format("\\%02X", byte) : byte.chr }.join
      end
    end
  end
end

module OpenSSL
  class BNError < OpenSSLError; end

  # An arbitrary-precision integer, as OpenSSL hands numbers to Ruby.
  class BN
    include Comparable

    def initialize(value = 0, base = 10)
      @value = case value
               when BN then value.to_i
               when Integer then value
               when String then base == 2 ? value.b.unpack1("H*").to_i(16) : value.to_i(base)
               else raise TypeError, "Cannot convert #{value.class} into OpenSSL::BN"
               end
    end

    def to_i
      @value
    end
    alias to_int to_i

    def to_bn
      self
    end

    def to_s(base = 10)
      return @value.to_s(16).upcase.then { |text| text.length.odd? ? "0#{text}" : text } if base == 16
      return OpenSSL::ASN1.__integer_bytes__(@value) if base == 2
      @value.to_s(base)
    end

    # The number is held in C, so the inspection names only the object.
    def inspect
      ::Kernel.instance_method(:to_s).bind_call(self)
    end

    def num_bits
      @value.bit_length
    end

    def num_bytes
      (num_bits + 7) / 8
    end

    def <=>(other)
      @value <=> (other.is_a?(BN) ? other.to_i : other)
    end

    def ==(other)
      other.respond_to?(:to_int) && @value == other.to_int
    end
    alias eql? ==

    def hash
      @value.hash
    end

    def coerce(other)
      [other, @value]
    end
  end
end

class Integer
  def to_bn
    OpenSSL::BN.new(self)
  end
end

module OpenSSL

  module ASN1
    # The big-endian bytes of a non-negative integer, with no leading zeros.
    def self.__integer_bytes__(value)
      return "\x00".b if value.zero?
      hex = value.to_s(16)
      hex = "0#{hex}" if hex.length.odd?
      [hex].pack("H*")
    end

    def self.__integer__(value)
      bytes = __integer_bytes__(value)
      bytes = "\x00".b + bytes if bytes.getbyte(0) >= 0x80
      __encoded__(INTEGER, bytes)
    end

    def self.__sequence__(*parts)
      __encoded__(0x30, parts.join.b)
    end

    def self.__object__(oid)
      __encoded__(OBJECT, OpenSSL.__object__(:encode, oid))
    end

    # A time as an X.509 validity bound: UTCTime through 2049, and
    # GeneralizedTime after.
    def self.__time__(time)
      held = time.utc
      if held.year < 2050
        __encoded__(UTCTIME, held.strftime("%y%m%d%H%M%SZ"))
      else
        __encoded__(GENERALIZEDTIME, held.strftime("%Y%m%d%H%M%SZ"))
      end
    end
  end

  module PKey
    class PKeyError < OpenSSL::OpenSSLError; end
    ECError = PKeyError
    RSAError = PKeyError

    class PKey; end

    # The key PEM or DER text holds: an RSA key or an EC one.
    def self.read(text, password = nil)
      text = text.read if text.respond_to?(:read)
      begin
        RSA.new(text, password)
      rescue PKeyError
        EC.new(text)
      end
    rescue PKeyError
      raise PKeyError, "Could not parse PKey"
    end

    # An elliptic curve key on one of the named prime curves: the public
    # point, and the private number that point is the generator times.
    class EC < PKey
      EC_OID = "1.2.840.10045.2.1"

      # Each named curve: its size in bits, the name NIST gives it, and its
      # parameters, the prime p of the field, the coefficients a and b of
      # y^2 = x^3 + ax + b, the generator's coordinates, and the generator's
      # order n. The cofactor is 1 where none is given.
      CURVES = {
        "prime256v1" => [256, "P-256", {
          p: 0xffffffff00000001000000000000000000000000ffffffffffffffffffffffff,
          a: 0xffffffff00000001000000000000000000000000fffffffffffffffffffffffc,
          b: 0x5ac635d8aa3a93e7b3ebbd55769886bc651d06b0cc53b0f63bce3c3e27d2604b,
          x: 0x6b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296,
          y: 0x4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5,
          n: 0xffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551
        }],
        "prime239v2" => [239, nil, {
          p: 0x7fffffffffffffffffffffff7fffffffffff8000000000007fffffffffff,
          a: 0x7fffffffffffffffffffffff7fffffffffff8000000000007ffffffffffc,
          b: 0x617fab6832576cbbfed50d99f0249c3fee58b94ba0038c7ae84c8c832f2c,
          x: 0x38af09d98727705120c921bb5e9e26296a3cdcf2f35757a0eafd87b830e7,
          y: 0x5b0125e4dbea0ec7206da0fc01d9b081329fb555de6ef460237dff8be4ba,
          n: 0x7fffffffffffffffffffffff800000cfa7e8594377d414c03821bc582063
        }],
        "secp256k1" => [256, nil, {
          p: 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f,
          a: 0,
          b: 7,
          x: 0x79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798,
          y: 0x483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8,
          n: 0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141
        }],
        "secp384r1" => [384, "P-384", {
          p: 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffeffffffff0000000000000000ffffffff,
          a: 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffeffffffff0000000000000000fffffffc,
          b: 0xb3312fa7e23ee7e4988e056be3f82d19181d9c6efe8141120314088f5013875ac656398d8a2ed19d2a85c8edd3ec2aef,
          x: 0xaa87ca22be8b05378eb1c71ef320ad746e1d3b628ba79b9859f741e082542a385502f25dbf55296c3a545e3872760ab7,
          y: 0x3617de4a96262c6f5d9e98bf9292dc29f8f41dbd289a147ce9da3113b5f0b8c00a60b1ce1d7e819d7a431d7c90ea0e5f,
          n: 0xffffffffffffffffffffffffffffffffffffffffffffffffc7634d81f4372ddf581a0db248b0a77aecec196accc52973
        }],
        "secp521r1" => [521, "P-521", {
          p: (1 << 521) - 1,
          a: (1 << 521) - 4,
          b: 0x0051953eb9618e1c9a1f929a21a0b68540eea2da725b99b315f3b8b489918ef109e156193951ec7e937b1652c0bd3bb1bf073573df883d2c34f1ef451fd46b503f00,
          x: 0x00c6858e06b70404e9cd9e3ecb662395b4429c648139053fb521f828af606b4d3dbaa14b5e77efe75928fe1dc127a2ffa8de3348b3c1856a429bf97e7e31c2e5bd66,
          y: 0x011839296a789a3bc0045c8a5fb42c7d1bd998f54449579b446817afbd17273e662c97ee72995ef42640c550b9013fad0761353c7086a272c24088be94769fd16650,
          n: 0x01fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffa51868783bf2f966b7fcc0148f709a5d03bb5c9b8899c47aebb6fb71e91386409
        }],
        # A curve over GF(2^571) rather than a prime field, its field
        # polynomial named by the exponents of its terms.
        "sect571r1" => [571, "B-571", {
          field: [571, 10, 5, 2, 0],
          cofactor: 2,
          a: 1,
          b: 0x02f40e7e2221f295de297117b7f3d62f5c6a97ffcb8ceff1cd6ba8ce4a9a18ad84ffabbd8efa59332be7ad6756a66e294afd185a78ff12aa520e4de739baca0c7ffeff7f2955727a,
          x: 0x0303001d34b856296c16c0d40d3cd7750a93d1d2955fa80aa5f40fc8db7b2abdbde53950f4c0d293cdd711a35b67fb1499ae60038614f1394abfa3b4c850d927e1e7769c8eec2d19,
          y: 0x037bf27342da639b6dccfffeb73d69d78c6c27a6009cbbca1980f8533921e8a684423e43bab08a576291af8f461bb2a8b3531d2f0485c19b16e2f1516e23dd3c1a4827af1b8ac15b,
          n: 0x03ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe661ce18ff55987308059b186823851ec7dd9ca1161de93d5174d66e8382e9bb2fe84e47
        }]
      }.freeze

      # The names OpenSSL also knows the NIST curves by.
      CURVE_ALIASES = { "P-256" => "prime256v1", "P-384" => "secp384r1", "P-521" => "secp521r1", "B-571" => "sect571r1" }.freeze

      # An operation on a binary curve's points, worked in the interpreter.
      # Coordinates go as hex and come back as Integers, and nil stands for
      # the point at infinity.
      def self.__binary__(group, operation, *values)
        parameters = group.__parameters__
        given = values.map { |value| value.is_a?(::Integer) ? value.to_s(16) : value }
        answer = OpenSSL.__ec2m__(operation, parameters[:field], parameters[:a].to_s(16), parameters[:b].to_s(16), *given)
        case answer
        when Array then answer.map { |held| held.to_i(16) }
        when String then answer.to_i(16)
        else answer
        end
      end

      def self.__binary_field__(group)
        group.__parameters__.key?(:field)
      end

      # The curves this knows, in the order OpenSSL lists them, with the
      # description it gives each.
      BUILTIN_CURVES = [
        ["secp256k1", "SECG curve over a 256 bit prime field"],
        ["secp384r1", "NIST/SECG curve over a 384 bit prime field"],
        ["secp521r1", "NIST/SECG curve over a 521 bit prime field"],
        ["prime239v2", "X9.62 curve over a 239 bit prime field"],
        ["prime256v1", "X9.62/SECG curve over a 256 bit prime field"],
        ["sect571r1", "NIST/SECG curve over a 571 bit binary field"]
      ].freeze

      def self.builtin_curves
        BUILTIN_CURVES.map(&:dup)
      end

      # The curve a key lies on, known by name.
      class Group
        class Error < OpenSSL::OpenSSLError; end

        attr_reader :curve_name

        def initialize(curve_name)
          curve_name = curve_name.curve_name if curve_name.is_a?(Group)
          named = CURVE_ALIASES.fetch(curve_name.to_s, curve_name.to_s)
          raise ECError, "unknown curve name (#{curve_name})" unless CURVES.key?(named)
          @curve_name = named
        end

        def degree
          CURVES[@curve_name][0]
        end

        def order
          OpenSSL::BN.new(__parameters__[:n])
        end

        def cofactor
          OpenSSL::BN.new(__parameters__.fetch(:cofactor, 1))
        end

        def generator
          Point.__affine__(self, [__parameters__[:x], __parameters__[:y]])
        end

        def point_conversion_form
          :uncompressed
        end

        def asn1_flag
          OpenSSL::PKey::EC::NAMED_CURVE
        end

        def ==(other)
          other.is_a?(Group) && other.curve_name == @curve_name
        end
        alias eql? ==

        def __parameters__
          CURVES[@curve_name][2]
        end

        # The bytes one coordinate takes.
        def __width__
          (degree + 7) / 8
        end
      end

      NAMED_CURVE = 1

      # A point on the curve, held by its affine coordinates, or as the point
      # at infinity.
      class Point
        class Error < OpenSSL::OpenSSLError; end

        attr_reader :group

        # The point SEC 1 octets name, given as a String or as the BN that
        # reads them.
        def initialize(group, encoded = nil)
          @group = group.is_a?(Group) ? group : Group.new(group)
          @coordinates = nil
          return if encoded.nil?
          octets = encoded.is_a?(OpenSSL::BN) ? encoded.to_s(2) : encoded.to_str.b
          @coordinates = Point.__decoded__(@group, octets)
          raise Error, "invalid encoding" if @coordinates == :invalid
        end

        # The point at `coordinates`, or at infinity for nil.
        def self.__affine__(group, coordinates)
          point = allocate
          point.instance_variable_set(:@group, group)
          point.instance_variable_set(:@coordinates, coordinates)
          point
        end

        # The coordinates SEC 1 octets name, nil for the point at infinity,
        # or :invalid.
        def self.__decoded__(group, octets)
          width = group.__width__
          form = octets.getbyte(0)
          return nil if form == 0 && octets.bytesize == 1
          prime = group.__parameters__[:p]
          case form
          when 4, 6, 7
            return :invalid unless octets.bytesize == 1 + 2 * width
            x = octets.byteslice(1, width).unpack1("H*").to_i(16)
            y = octets.byteslice(1 + width, width).unpack1("H*").to_i(16)
          when 2, 3
            return :invalid unless octets.bytesize == 1 + width
            x = octets.byteslice(1, width).unpack1("H*").to_i(16)
            if EC.__binary_field__(group)
              y = EC.__binary__(group, "decompress", x, form - 2)
              return y.nil? ? :invalid : [x, y]
            end
            parameters = group.__parameters__
            square = (x.pow(3, prime) + parameters[:a] * x + parameters[:b]) % prime
            y = square.pow((prime + 1) / 4, prime)
            return :invalid unless y.pow(2, prime) == square
            y = prime - y if y.odd? != (form == 3)
          else
            return :invalid
          end
          return :invalid unless EC.__on_curve__(group, x, y)
          [x, y]
        end

        def infinity?
          @coordinates.nil?
        end

        def on_curve?
          infinity? || EC.__on_curve__(@group, *@coordinates)
        end

        def to_octet_string(form)
          return "\x00".b if infinity?
          width = @group.__width__
          x, y = @coordinates
          x_bytes = OpenSSL::ASN1.__integer_bytes__(x).rjust(width, "\x00".b)
          odd = EC.__binary_field__(@group) ? EC.__binary__(@group, "y_bit", x, y) == 1 : y.odd?
          case form
          when :compressed then (odd ? "\x03" : "\x02").b + x_bytes
          when :hybrid then (odd ? "\x07" : "\x06").b + x_bytes + OpenSSL::ASN1.__integer_bytes__(y).rjust(width, "\x00".b)
          else "\x04".b + x_bytes + OpenSSL::ASN1.__integer_bytes__(y).rjust(width, "\x00".b)
          end
        end

        def to_bn(form = :uncompressed)
          OpenSSL::BN.new(to_octet_string(form), 2)
        end

        # The point times `number`, plus the generator times `base` when
        # one is given.
        def mul(number, base = nil)
          product = EC.__multiply__(@group, @coordinates, number.to_i)
          if base
            product = EC.__add__(@group, product, EC.__multiply__(@group, @group.generator.__coordinates__, base.to_i))
          end
          Point.__affine__(@group, product)
        end

        def add(other)
          Point.__affine__(@group, EC.__add__(@group, @coordinates, other.__coordinates__))
        end

        def __coordinates__
          @coordinates
        end

        def ==(other)
          other.is_a?(Point) && other.group == @group && other.__coordinates__ == @coordinates
        end
        alias eql? ==
      end

      def self.__on_curve__(group, x, y)
        if __binary_field__(group)
          bound = 1 << group.degree
          return x.between?(0, bound - 1) && y.between?(0, bound - 1) && __binary__(group, "on_curve", x, y)
        end
        parameters = group.__parameters__
        prime = parameters[:p]
        return false unless x.between?(0, prime - 1) && y.between?(0, prime - 1)
        (y * y - (x.pow(3, prime) + parameters[:a] * x + parameters[:b])) % prime == 0
      end

      # Points in Jacobian coordinates, X/Z^2 and Y/Z^3, so that adding and
      # doubling need no inverse until the end. nil is the point at infinity.
      def self.__doubled__(parameters, held)
        return nil if held.nil?
        prime = parameters[:p]
        x, y, z = held
        return nil if y.zero?
        y_squared = y * y % prime
        s = 4 * x * y_squared % prime
        z_squared = z * z % prime
        m = (3 * x * x + parameters[:a] * z_squared * z_squared) % prime
        new_x = (m * m - 2 * s) % prime
        new_y = (m * (s - new_x) - 8 * y_squared * y_squared) % prime
        [new_x, new_y, 2 * y * z % prime]
      end

      def self.__summed__(parameters, first, second)
        return second if first.nil?
        return first if second.nil?
        prime = parameters[:p]
        x1, y1, z1 = first
        x2, y2, z2 = second
        z1_squared = z1 * z1 % prime
        z2_squared = z2 * z2 % prime
        u1 = x1 * z2_squared % prime
        u2 = x2 * z1_squared % prime
        s1 = y1 * z2_squared * z2 % prime
        s2 = y2 * z1_squared * z1 % prime
        if u1 == u2
          return s1 == s2 ? __doubled__(parameters, first) : nil
        end
        h = (u2 - u1) % prime
        r = (s2 - s1) % prime
        h_squared = h * h % prime
        h_cubed = h_squared * h % prime
        new_x = (r * r - h_cubed - 2 * u1 * h_squared) % prime
        new_y = (r * (u1 * h_squared - new_x) - s1 * h_cubed) % prime
        [new_x, new_y, h * z1 * z2 % prime]
      end

      def self.__affine_of__(parameters, held)
        return nil if held.nil?
        prime = parameters[:p]
        x, y, z = held
        inverse = z.pow(prime - 2, prime)
        inverse_squared = inverse * inverse % prime
        [x * inverse_squared % prime, y * inverse_squared * inverse % prime]
      end

      # `coordinates` times `number`, by doubling and adding over its bits.
      def self.__multiply__(group, coordinates, number)
        return nil if coordinates.nil?
        parameters = group.__parameters__
        number %= parameters[:n]
        return __binary__(group, "multiply", *coordinates, number) if __binary_field__(group)
        point = [coordinates[0], coordinates[1], 1]
        total = nil
        (number.bit_length - 1).downto(0) do |bit|
          total = __doubled__(parameters, total)
          total = __summed__(parameters, total, point) if number[bit] == 1
        end
        __affine_of__(parameters, total)
      end

      def self.__add__(group, first, second)
        if __binary_field__(group)
          return __binary__(group, "add", *(first || [nil, nil]), *(second || [nil, nil]))
        end
        parameters = group.__parameters__
        lifted = [first, second].map { |held| held && [held[0], held[1], 1] }
        __affine_of__(parameters, __summed__(parameters, *lifted))
      end

      attr_reader :group

      # A key read from PEM or DER text, an empty key on the curve a name or
      # Group gives, or an empty key with no curve at all.
      def initialize(source = nil, password = nil)
        @group = nil
        @private_key = nil
        @public_key = nil
        return if source.nil?
        if source.is_a?(Group)
          @group = source
          return
        end
        source = source.read if source.respond_to?(:read)
        source = source.to_str
        if CURVES.key?(CURVE_ALIASES.fetch(source, source))
          @group = Group.new(source)
          return
        end
        parts = EC.__read__(source)
        raise ECError, "invalid curve name" if parts.nil?
        curve, point_octets, secret = parts
        @group = Group.new(curve)
        @private_key = secret && OpenSSL::BN.new(secret)
        @public_key = point_octets ? Point.new(@group, point_octets) : (secret && @group.generator.mul(secret))
      end

      # A new key pair on the curve named.
      def self.generate(curve)
        key = new(curve.is_a?(Group) ? curve : Group.new(curve))
        key.generate_key!
      end

      def generate_key!
        raise ECError, "no curve" if @group.nil?
        order = @group.__parameters__[:n]
        secret = 0
        secret = OpenSSL::Random.random_bytes(@group.__width__ + 8).unpack1("H*").to_i(16) % order until secret.positive?
        @private_key = OpenSSL::BN.new(secret)
        @public_key = @group.generator.mul(secret)
        self
      end
      alias generate_key generate_key!

      # The curve, the public point, and the private number of a key, from a
      # SubjectPublicKeyInfo, a SEC 1 ECPrivateKey, or a PKCS #8 key, in PEM
      # or DER.
      def self.__read__(text)
        der = text.b
        if text.include?("-----BEGIN ")
          matched = text.match(/-----BEGIN (PUBLIC KEY|EC PRIVATE KEY|PRIVATE KEY)-----(.*?)-----END \1-----/m)
          return nil if matched.nil?
          der = matched[2].unpack1("m")
        end
        top = OpenSSL::ASN1.__read_der__(der)
        return nil unless top && top[0] == 0x30
        fields = OpenSSL::ASN1.__read_all__(top[1])
        return nil if fields.nil? || fields.size < 2
        if fields[0][0] == 0x30 && fields[1][0] == OpenSSL::ASN1::BIT_STRING
          curve = __algorithm_curve__(fields[0][1])
          return curve && [curve, fields[1][1].byteslice(1..), nil]
        end
        if fields[0][0] == OpenSSL::ASN1::INTEGER && fields[1][0] == 0x30 && fields[2]&.first == OpenSSL::ASN1::OCTET_STRING
          curve = __algorithm_curve__(fields[1][1])
          return nil if curve.nil?
          inner = __read__(fields[2][1])
          return inner && [curve, inner[1], inner[2]]
        end
        return nil unless fields[0][0] == OpenSSL::ASN1::INTEGER && fields[1][0] == OpenSSL::ASN1::OCTET_STRING
        secret = fields[1][1].unpack1("H*").to_i(16)
        curve = nil
        point = nil
        fields.drop(2).each do |tag, content|
          inner = OpenSSL::ASN1.__read_der__(content)
          next if inner.nil?
          if tag == 0xa0 && inner[0] == OpenSSL::ASN1::OBJECT
            found = OpenSSL.__object__(:find, OpenSSL::ASN1.__oid_text__(inner[1]))
            curve = found && found[0]
          elsif tag == 0xa1 && inner[0] == OpenSSL::ASN1::BIT_STRING
            point = inner[1].byteslice(1..)
          end
        end
        [curve, point, secret]
      end

      # The named curve an AlgorithmIdentifier for an EC key gives.
      def self.__algorithm_curve__(content)
        named, parameters = OpenSSL::ASN1.__read_all__(content)
        return nil unless named && parameters && parameters[0] == OpenSSL::ASN1::OBJECT
        return nil unless OpenSSL::ASN1.__oid_text__(named[1]) == EC_OID
        found = OpenSSL.__object__(:find, OpenSSL::ASN1.__oid_text__(parameters[1]))
        found && CURVES.key?(found[0]) ? found[0] : nil
      end

      def oid
        "id-ecPublicKey"
      end

      def public_key
        @public_key
      end

      def public_key=(point)
        raise TypeError, "wrong argument type #{point.class} (expected OpenSSL/EC_POINT)" unless point.nil? || point.is_a?(Point)
        @public_key = point
      end

      def private_key
        @private_key
      end

      def private_key=(number)
        @private_key = number.nil? ? nil : OpenSSL::BN.new(number.to_i)
      end

      def public?
        !@public_key.nil?
      end

      def private?
        !@private_key.nil?
      end
      alias private_key? private?
      alias public_key? public?

      def check_key
        raise ECError, "EC_KEY_check_key: point is not on curve" unless @public_key&.on_curve?
        if @private_key && @group.generator.mul(@private_key) != @public_key
          raise ECError, "EC_KEY_check_key: invalid private key"
        end
        true
      end

      # The leftmost bits of a digest, as many as the order has.
      def __digest_number__(digest)
        number = digest.b.unpack1("H*").to_i(16)
        excess = digest.bytesize * 8 - @group.__parameters__[:n].bit_length
        excess.positive? ? number >> excess : number
      end
      private :__digest_number__

      # An ECDSA signature of a digest, as the DER SEQUENCE of r and s.
      def dsa_sign_asn1(digest)
        raise ECError, "ECDSA_sign: missing private key" unless private?
        order = @group.__parameters__[:n]
        number = __digest_number__(digest.to_str)
        secret = @private_key.to_i
        loop do
          nonce = OpenSSL::Random.random_bytes(@group.__width__ + 8).unpack1("H*").to_i(16) % order
          next if nonce.zero?
          x, = EC.__multiply__(@group, @group.generator.__coordinates__, nonce)
          r = x % order
          next if r.zero?
          s = nonce.pow(order - 2, order) * (number + r * secret) % order
          next if s.zero?
          return OpenSSL::ASN1.__sequence__(OpenSSL::ASN1.__integer__(r), OpenSSL::ASN1.__integer__(s))
        end
      end

      def dsa_verify_asn1(digest, signature)
        raise ECError, "ECDSA_verify: missing public key" unless public?
        top = OpenSSL::ASN1.__read_der__(signature.to_str.b)
        numbers = top && top[0] == 0x30 && top[2].empty? && OpenSSL::ASN1.__read_all__(top[1])
        unless numbers && numbers.size == 2 && numbers.all? { |tag, _| tag == OpenSSL::ASN1::INTEGER }
          raise ECError, "ECDSA_verify: nested asn1 error"
        end
        r, s = numbers.map { |_, content| OpenSSL::ASN1.__signed_value__(content) }
        order = @group.__parameters__[:n]
        return false unless r.between?(1, order - 1) && s.between?(1, order - 1)
        inverse = s.pow(order - 2, order)
        number = __digest_number__(digest.to_str)
        first = EC.__multiply__(@group, @group.generator.__coordinates__, number * inverse % order)
        second = EC.__multiply__(@group, @public_key.__coordinates__, r * inverse % order)
        total = EC.__add__(@group, first, second)
        !total.nil? && total[0] % order == r
      end

      def sign(digest, data)
        dsa_sign_asn1(__hashed__(digest, data))
      end

      def verify(digest, signature, data)
        dsa_verify_asn1(__hashed__(digest, data), signature)
      rescue ECError
        false
      end

      def __hashed__(digest, data)
        named = OpenSSL::Digest.canonical_name(digest.respond_to?(:name) ? digest.name : digest.to_s)
        OpenSSL::Digest.digest(named, data).b
      end
      private :__hashed__

      def __curve_object__
        OpenSSL::ASN1.__object__(OpenSSL.__object__(:find, @group.curve_name)[2])
      end
      private :__curve_object__

      def __point_bits__
        OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::BIT_STRING, "\x00".b + @public_key.to_octet_string(:uncompressed))
      end
      private :__point_bits__

      def public_to_der
        algorithm = OpenSSL::ASN1.__sequence__(OpenSSL::ASN1.__object__(EC_OID), __curve_object__)
        OpenSSL::ASN1.__sequence__(algorithm, __point_bits__)
      end

      def public_to_pem
        "-----BEGIN PUBLIC KEY-----\n#{[public_to_der].pack("m48")}-----END PUBLIC KEY-----\n"
      end

      # The SEC 1 ECPrivateKey, with the curve when `named` asks for it.
      def __private_key_der__(named)
        secret = OpenSSL::ASN1.__integer_bytes__(@private_key.to_i).rjust(@group.__width__, "\x00".b)
        parts = [OpenSSL::ASN1.__integer__(1), OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::OCTET_STRING, secret)]
        parts << OpenSSL::ASN1.__encoded__(0xa0, __curve_object__) if named
        parts << OpenSSL::ASN1.__encoded__(0xa1, __point_bits__) if public?
        OpenSSL::ASN1.__sequence__(*parts)
      end
      private :__private_key_der__

      def private_to_der
        raise ECError, "private key is needed" unless private?
        algorithm = OpenSSL::ASN1.__sequence__(OpenSSL::ASN1.__object__(EC_OID), __curve_object__)
        inner = OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::OCTET_STRING, __private_key_der__(false))
        OpenSSL::ASN1.__sequence__(OpenSSL::ASN1.__integer__(0), algorithm, inner)
      end

      def private_to_pem
        "-----BEGIN PRIVATE KEY-----\n#{[private_to_der].pack("m48")}-----END PRIVATE KEY-----\n"
      end

      def to_der
        private? ? __private_key_der__(true) : public_to_der
      end

      def to_pem
        return public_to_pem unless private?
        "-----BEGIN EC PRIVATE KEY-----\n#{[to_der].pack("m48")}-----END EC PRIVATE KEY-----\n"
      end
      alias to_s to_pem
      alias export to_pem

      # The bytes of the public key a key identifier hashes: the point itself.
      def __public_key_bits__
        @public_key.to_octet_string(:uncompressed)
      end

      # The lines X509_print writes for the key under Subject Public Key Info.
      def __printed_lines__
        rows = __public_key_bits__.unpack1("H*").scan(/../).each_slice(15).map { |row| row.join(":") }
        nist = CURVES[@group.curve_name][1]
        [
          "            Public Key Algorithm: id-ecPublicKey",
          "                Public-Key: (#{@group.degree} bit)",
          "                pub:",
          *rows.each_with_index.map { |row, at| "                    #{row}#{":" if at < rows.size - 1}" },
          "                ASN1 OID: #{@group.curve_name}",
          *(nist ? ["                NIST CURVE: #{nist}"] : [])
        ]
      end
    end

    # An RSA key pair, or the public half of one.
    class RSA < PKey
      RSA_OID = "1.2.840.113549.1.1.1"

      # The DigestInfo prefix PKCS #1 v1.5 puts ahead of each digest.
      DIGEST_INFO = {
        "SHA1" => ["3021300906052b0e03021a05000414"].pack("H*"),
        "SHA256" => ["3031300d060960864801650304020105000420"].pack("H*"),
        "SHA384" => ["3041300d060960864801650304020205000430"].pack("H*"),
        "SHA512" => ["3051300d060960864801650304020305000440"].pack("H*")
      }.freeze

      def self.generate(bits, exponent = 65_537)
        new(bits, exponent)
      end

      # A key generated at the size given, or read from the PEM or DER text
      # of one.
      def initialize(bits = nil, exponent = 65_537)
        return if bits.nil?
        # A File holding a key is read for its text, as a String would be.
        bits = bits.read if bits.is_a?(File)
        bits = bits.to_str if !bits.is_a?(String) && !bits.is_a?(Integer) && bits.respond_to?(:to_str)
        unless bits.is_a?(String) || bits.is_a?(Integer)
          raise TypeError, "no implicit conversion of #{bits.class} into String"
        end
        if bits.is_a?(String)
          # Read from text, the second argument is the password of a key
          # written encrypted.
          password = exponent.is_a?(Integer) ? nil : exponent
          password = password.to_str if !password.nil? && password.respond_to?(:to_str)
          parts = RSA.__read__(bits, password)
          raise PKeyError, "Neither PUB key nor PRIV key" if parts.nil?
          @n, @e, @d, @p, @q, @dmp1, @dmq1, @iqmp = parts
          return
        end
        if bits < 512
          raise PKeyError, "EVP_PKEY_CTX_ctrl_str(ctx, \"rsa_keygen_bits\", \"#{bits}\"): key size too small"
        end
        if exponent.negative?
          raise PKeyError, "EVP_PKEY_CTX_ctrl_str(ctx, \"rsa_keygen_pubexp\", \"#{exponent}\"): invalid negative value"
        end
        raise PKeyError, "EVP_PKEY_keygen: pub exponent out of range" if exponent < 3 || exponent.even?
        @n, @e, @d, @p, @q, @dmp1, @dmq1, @iqmp = OpenSSL.__rsa_generate__(bits, exponent)
      end

      PEM_LABELS = ["RSA PRIVATE KEY", "PUBLIC KEY", "RSA PUBLIC KEY", "PRIVATE KEY", "ENCRYPTED PRIVATE KEY"].freeze

      PBES2_OID = "1.2.840.113549.1.5.13"
      PBKDF2_OID = "1.2.840.113549.1.5.12"
      # The ciphers PBES2 names, with the size of key each takes.
      PBES2_CIPHERS = {
        "2.16.840.1.101.3.4.1.2" => ["aes-128-cbc", 16],
        "2.16.840.1.101.3.4.1.22" => ["aes-192-cbc", 24],
        "2.16.840.1.101.3.4.1.42" => ["aes-256-cbc", 32]
      }.freeze
      # The HMAC digests PBKDF2 names, SHA-1 when it names none.
      PBKDF2_DIGESTS = {
        "1.2.840.113549.2.7" => "SHA1",
        "1.2.840.113549.2.8" => "SHA224",
        "1.2.840.113549.2.9" => "SHA256",
        "1.2.840.113549.2.10" => "SHA384",
        "1.2.840.113549.2.11" => "SHA512"
      }.freeze

      # The parts of a key written as PKCS #1 RSAPrivateKey, as PKCS #8, plain
      # or encrypted with a password, as a SubjectPublicKeyInfo, or as PKCS #1
      # RSAPublicKey, in DER or in PEM.
      def self.__read__(text, password = nil)
        der = text.b
        if text.include?("-----BEGIN ")
          matched = text.match(/-----BEGIN (#{PEM_LABELS.join("|")})-----(.*?)-----END \1-----/m)
          return nil if matched.nil?
          body = matched[2]
          if body.include?("DEK-Info:")
            der = __legacy_decrypted__(body, password)
            return nil if der.nil?
          else
            der = body.unpack1("m")
          end
          if matched[1] == "ENCRYPTED PRIVATE KEY"
            der = __pkcs8_decrypted__(der, password)
            return nil if der.nil?
          end
        end
        top = OpenSSL::ASN1.__read_der__(der)
        return nil unless top && top[0] == 0x30
        fields = OpenSSL::ASN1.__read_all__(top[1])
        return nil if fields.nil?
        # PKCS #8 PrivateKeyInfo: a version, the algorithm, then the PKCS #1
        # key inside an OCTET STRING.
        if fields.size >= 3 && fields[0][0] == OpenSSL::ASN1::INTEGER && fields[1][0] == 0x30 &&
           fields[2][0] == OpenSSL::ASN1::OCTET_STRING
          algorithm = OpenSSL::ASN1.__read_all__(fields[1][1])
          return nil unless algorithm && OpenSSL::ASN1.__oid_text__(algorithm[0][1]) == RSA_OID
          return __read__(fields[2][1])
        end
        if fields.size == 9 && fields.all? { |tag, _| tag == OpenSSL::ASN1::INTEGER }
          return fields.drop(1).map { |_, content| OpenSSL::ASN1.__integer_value__(content) }
        end
        if fields.size == 2 && fields.all? { |tag, _| tag == OpenSSL::ASN1::INTEGER }
          return fields.map { |_, content| OpenSSL::ASN1.__integer_value__(content) }
        end
        if fields.size == 2 && fields[0][0] == 0x30 && fields[1][0] == OpenSSL::ASN1::BIT_STRING
          inner = OpenSSL::ASN1.__read_der__(fields[1][1].byteslice(1..))
          numbers = inner && OpenSSL::ASN1.__read_all__(inner[1])
          return nil unless numbers && numbers.size == 2 && numbers.all? { |tag, _| tag == OpenSSL::ASN1::INTEGER }
          return numbers.map { |_, content| OpenSSL::ASN1.__integer_value__(content) }
        end
        nil
      end

      # EncryptedPrivateKeyInfo under PBES2: PBKDF2 makes the key from the
      # password, and the cipher it names opens the PrivateKeyInfo.
      def self.__pkcs8_decrypted__(der, password)
        return nil if password.nil?
        top = OpenSSL::ASN1.__read_der__(der)
        algorithm, encrypted = OpenSSL::ASN1.__read_all__(top[1])
        scheme, parameters = OpenSSL::ASN1.__read_all__(algorithm[1])
        return nil unless OpenSSL::ASN1.__oid_text__(scheme[1]) == PBES2_OID
        derivation, encryption = OpenSSL::ASN1.__read_all__(parameters[1])
        kdf, kdf_parameters = OpenSSL::ASN1.__read_all__(derivation[1])
        return nil unless OpenSSL::ASN1.__oid_text__(kdf[1]) == PBKDF2_OID
        salt, iterations, *rest = OpenSSL::ASN1.__read_all__(kdf_parameters[1])
        prf = rest.find { |tag, _| tag == 0x30 }
        digest = prf ? PBKDF2_DIGESTS[OpenSSL::ASN1.__oid_text__(OpenSSL::ASN1.__read_all__(prf[1])[0][1])] : "SHA1"
        cipher_oid, iv = OpenSSL::ASN1.__read_all__(encryption[1])
        cipher_name, key_length = PBES2_CIPHERS[OpenSSL::ASN1.__oid_text__(cipher_oid[1])]
        return nil if digest.nil? || cipher_name.nil?
        key = OpenSSL::KDF.pbkdf2_hmac(password, salt: salt[1], iterations: OpenSSL::ASN1.__integer_value__(iterations[1]),
                                                 length: key_length, hash: digest)
        __decrypted__(cipher_name, key, iv[1], encrypted[1])
      end

      # A PEM body OpenSSL encrypted the older way: the DEK-Info header names
      # the cipher and IV, and the key comes from MD5 over the password and
      # the IV's first eight bytes.
      def self.__legacy_decrypted__(body, password)
        return nil if password.nil?
        headers, encoded = body.split(/\r?\n\r?\n/, 2)
        info = headers[/^DEK-Info: *(.+)$/, 1]
        return nil if info.nil? || encoded.nil?
        cipher_name, iv_hex = info.strip.split(",", 2)
        cipher_name = cipher_name.downcase
        iv = [iv_hex].pack("H*")
        cipher = OpenSSL::Cipher.new(cipher_name)
        salt = iv.byteslice(0, 8)
        key = "".b
        block = "".b
        while key.bytesize < cipher.key_len
          block = OpenSSL::Digest.digest("MD5", block + password.b + salt).b
          key << block
        end
        __decrypted__(cipher_name, key.byteslice(0, cipher.key_len), iv, encoded.unpack1("m"))
      rescue OpenSSL::Cipher::CipherError
        nil
      end

      def self.__decrypted__(cipher_name, key, iv, encrypted)
        cipher = OpenSSL::Cipher.new(cipher_name)
        cipher.decrypt
        cipher.key = key
        cipher.iv = iv
        cipher.update(encrypted) + cipher.final
      rescue OpenSSL::Cipher::CipherError
        nil
      end

      def to_pem
        label = private? ? "RSA PRIVATE KEY" : "PUBLIC KEY"
        "-----BEGIN #{label}-----\n#{[to_der].pack("m48")}-----END #{label}-----\n"
      end
      alias to_s to_pem
      alias export to_pem

      def public_to_pem
        "-----BEGIN PUBLIC KEY-----\n#{[public_to_der].pack("m48")}-----END PUBLIC KEY-----\n"
      end

      def self.__from_parts__(parts)
        made = allocate
        made.instance_variable_set(:@n, parts[0])
        made.instance_variable_set(:@e, parts[1])
        made
      end

      %i[n e d p q dmp1 dmq1 iqmp].each do |part|
        define_method(part) do
          held = instance_variable_get(:"@#{part}")
          held.nil? ? nil : OpenSSL::BN.new(held)
        end
      end

      def private?
        !@d.nil?
      end

      def public?
        !@n.nil?
      end

      def public_key
        RSA.__from_parts__([@n, @e])
      end

      # SubjectPublicKeyInfo: the algorithm, then the key itself as a
      # SEQUENCE of the modulus and the public exponent.
      def public_to_der
        key = OpenSSL::ASN1.__sequence__(OpenSSL::ASN1.__integer__(@n), OpenSSL::ASN1.__integer__(@e))
        algorithm = OpenSSL::ASN1.__sequence__(OpenSSL::ASN1.__object__(RSA_OID), OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::NULL, ""))
        OpenSSL::ASN1.__sequence__(algorithm, OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::BIT_STRING, "\x00".b + key))
      end

      # The lines X509_print writes for the key under Subject Public Key Info.
      def __printed_lines__
        modulus = OpenSSL::ASN1.__integer_bytes__(@n)
        modulus = "\x00".b + modulus if modulus.getbyte(0) >= 0x80
        rows = modulus.unpack1("H*").scan(/../).each_slice(15).map { |row| row.join(":") }
        [
          "            Public Key Algorithm: rsaEncryption",
          "                Public-Key: (#{@n.bit_length} bit)",
          "                Modulus:",
          *rows.each_with_index.map { |row, at| "                    #{row}#{":" if at < rows.size - 1}" },
          "                Exponent: #{@e} (0x#{@e.to_s(16)})"
        ]
      end

      # The bytes of the public key a key identifier hashes: the key's own
      # SEQUENCE of modulus and exponent.
      def __public_key_bits__
        OpenSSL::ASN1.__sequence__(OpenSSL::ASN1.__integer__(@n), OpenSSL::ASN1.__integer__(@e))
      end

      def to_der
        return public_to_der unless private?
        OpenSSL::ASN1.__sequence__(*[0, @n, @e, @d, @p, @q, @dmp1, @dmq1, @iqmp].map { |part| OpenSSL::ASN1.__integer__(part) })
      end

      def sign(digest, data)
        raise ArgumentError, "private key is needed" unless private?
        hash, prefix = __digested__(digest, data)
        width = (@n.bit_length + 7) / 8
        padding = width - prefix.bytesize - hash.bytesize - 3
        raise PKeyError, "EVP_DigestSign: RSA lib" if padding < 8
        encoded = "\x00\x01".b + ("\xFF".b * padding) + "\x00".b + prefix + hash
        __private_power__(encoded, width)
      end

      # The private operation on `encoded`, in halves joined by the Chinese
      # remainder theorem, as `width` bytes.
      def __private_power__(encoded, width)
        message = encoded.unpack1("H*").to_i(16)
        first = message.pow(@dmp1, @p)
        second = message.pow(@dmq1, @q)
        joined = second + ((@iqmp * (first - second)) % @p) * @q
        OpenSSL::ASN1.__integer_bytes__(joined).rjust(width, "\x00".b)
      end
      private :__private_power__

      # An RSASSA-PSS signature. The salt is as long as the digest for
      # `:digest`, as long as the key leaves room for with `:max`, or the
      # number given.
      def sign_pss(digest, data, salt_length:, mgf1_hash:)
        raise ArgumentError, "private key is needed" unless private?
        hash_name = __digest_name__(digest)
        hashed = OpenSSL::Digest.digest(hash_name, data).b
        encoded_bits = @n.bit_length - 1
        encoded_length = (encoded_bits + 7) / 8
        salt_bytes = case salt_length
                     when :digest then hashed.bytesize
                     when :max then encoded_length - hashed.bytesize - 2
                     else __salt_bytes__(salt_length)
                     end
        if encoded_length < hashed.bytesize + salt_bytes + 2
          raise PKeyError, "EVP_PKEY_CTX_set_rsa_pss_saltlen: data too large for key size"
        end
        salt = OpenSSL::Random.random_bytes(salt_bytes).b
        check = OpenSSL::Digest.digest(hash_name, ("\x00".b * 8) + hashed + salt).b
        block = ("\x00".b * (encoded_length - salt_bytes - hashed.bytesize - 2)) + "\x01".b + salt
        masked = __xor__(block, __mgf1__(__digest_name__(mgf1_hash), check, block.bytesize))
        masked.setbyte(0, masked.getbyte(0) & (0xff >> (8 * encoded_length - encoded_bits)))
        __private_power__(masked + check + "\xBC".b, (@n.bit_length + 7) / 8)
      end

      def verify_pss(digest, signature, data, salt_length:, mgf1_hash:)
        hash_name = __digest_name__(digest)
        hashed = OpenSSL::Digest.digest(hash_name, data).b
        width = (@n.bit_length + 7) / 8
        return false unless signature.bytesize == width
        encoded_bits = @n.bit_length - 1
        encoded_length = (encoded_bits + 7) / 8
        recovered = signature.b.unpack1("H*").to_i(16).pow(@e, @n)
        return false if recovered.bit_length > encoded_bits
        encoded = OpenSSL::ASN1.__integer_bytes__(recovered).rjust(encoded_length, "\x00".b)
        return false unless encoded.getbyte(-1) == 0xbc && encoded_length >= hashed.bytesize + 2
        masked = encoded.byteslice(0, encoded_length - hashed.bytesize - 1)
        check = encoded.byteslice(masked.bytesize, hashed.bytesize)
        block = __xor__(masked, __mgf1__(__digest_name__(mgf1_hash), check, masked.bytesize))
        block.setbyte(0, block.getbyte(0) & (0xff >> (8 * encoded_length - encoded_bits)))
        start = block.index("\x01".b)
        return false if start.nil? || block.byteslice(0, start).bytes.any?(&:positive?)
        salt = block.byteslice(start + 1, block.bytesize - start - 1)
        expected = case salt_length
                   when :digest then hashed.bytesize
                   when :auto then salt.bytesize
                   else __salt_bytes__(salt_length)
                   end
        return false unless salt.bytesize == expected
        OpenSSL::Digest.digest(hash_name, ("\x00".b * 8) + hashed + salt).b == check
      end

      def __salt_bytes__(salt_length)
        return salt_length if salt_length.is_a?(Integer)
        raise TypeError, "no implicit conversion of #{salt_length.class} into Integer"
      end
      private :__salt_bytes__

      def __digest_name__(digest)
        OpenSSL::Digest.canonical_name(digest.respond_to?(:name) ? digest.name : digest.to_s)
      end
      private :__digest_name__

      # MGF1: the digests of `seed` and a counter, run together to `length`.
      def __mgf1__(hash_name, seed, length)
        mask = "".b
        counter = 0
        while mask.bytesize < length
          mask << OpenSSL::Digest.digest(hash_name, seed + [counter].pack("N")).b
          counter += 1
        end
        mask.byteslice(0, length)
      end
      private :__mgf1__

      def __xor__(first, second)
        first.bytes.zip(second.bytes).map { |left, right| left ^ right }.pack("C*")
      end
      private :__xor__

      def verify(digest, signature, data)
        hash, prefix = __digested__(digest, data)
        width = (@n.bit_length + 7) / 8
        return false unless signature.bytesize == width
        recovered = signature.b.unpack1("H*").to_i(16).pow(@e, @n)
        bytes = OpenSSL::ASN1.__integer_bytes__(recovered).rjust(width, "\x00".b)
        padding = width - prefix.bytesize - hash.bytesize - 3
        padding >= 8 && bytes == "\x00\x01".b + ("\xFF".b * padding) + "\x00".b + prefix + hash
      end

      def __digested__(digest, data)
        named = OpenSSL::Digest.canonical_name(digest.respond_to?(:name) ? digest.name : digest.to_s)
        prefix = DIGEST_INFO[named]
        raise PKeyError, "unsupported digest #{named}" if prefix.nil?
        [OpenSSL::Digest.digest(named, data).b, prefix]
      end
      private :__digested__
    end
  end
end

module OpenSSL
  module ASN1
    class ASN1Error < OpenSSL::OpenSSLError; end
  end

  module X509
    class ExtensionError < OpenSSL::OpenSSLError; end
    class CertificateError < OpenSSL::OpenSSLError; end
    class StoreError < OpenSSL::OpenSSLError; end

    V_OK = 0
    V_ERR_CERT_SIGNATURE_FAILURE = 7
    V_ERR_CERT_NOT_YET_VALID = 9
    V_ERR_CERT_HAS_EXPIRED = 10
    V_ERR_DEPTH_ZERO_SELF_SIGNED_CERT = 18
    V_ERR_SELF_SIGNED_CERT_IN_CHAIN = 19
    V_ERR_UNABLE_TO_GET_ISSUER_CERT_LOCALLY = 20
    V_ERR_UNABLE_TO_GET_ISSUER_CERT = 2
    V_ERR_UNABLE_TO_VERIFY_LEAF_SIGNATURE = 21
    V_ERR_CERT_REJECTED = 28
    V_ERR_INVALID_CA = 79
    V_ERR_HOSTNAME_MISMATCH = 62
    V_ERR_UNABLE_TO_GET_CRL = 3
    V_ERR_CRL_SIGNATURE_FAILURE = 8
    V_ERR_CRL_NOT_YET_VALID = 11
    V_ERR_CRL_HAS_EXPIRED = 12
    V_ERR_CERT_REVOKED = 23

    V_FLAG_CRL_CHECK = 0x4
    V_FLAG_CRL_CHECK_ALL = 0x8

    PURPOSE_SSL_CLIENT = 1
    PURPOSE_SSL_SERVER = 2
    PURPOSE_NS_SSL_SERVER = 3
    PURPOSE_SMIME_SIGN = 4
    PURPOSE_SMIME_ENCRYPT = 5
    PURPOSE_CRL_SIGN = 6
    PURPOSE_ANY = 7
    PURPOSE_OCSP_HELPER = 8
    PURPOSE_TIMESTAMP_SIGN = 9

    VERIFY_ERROR_STRINGS = {
      0 => "ok",
      7 => "certificate signature failure",
      2 => "unable to get issuer certificate",
      9 => "certificate is not yet valid",
      10 => "certificate has expired",
      18 => "self-signed certificate",
      19 => "self-signed certificate in certificate chain",
      20 => "unable to get local issuer certificate",
      21 => "unable to verify the first certificate",
      28 => "certificate rejected",
      79 => "invalid CA certificate",
      62 => "hostname mismatch",
      3 => "unable to get certificate CRL",
      8 => "CRL signature failure",
      11 => "CRL is not yet valid",
      12 => "CRL has expired",
      23 => "certificate revoked"
    }.freeze

    # The reasons a CRL entry may give for a revocation.
    CRL_REASONS = {
      0 => "Unspecified", 1 => "Key Compromise", 2 => "CA Compromise", 3 => "Affiliation Changed",
      4 => "Superseded", 5 => "Cessation Of Operation", 6 => "Certificate Hold",
      8 => "Remove From CRL", 9 => "Privilege Withdrawn", 10 => "AA Compromise"
    }.freeze

    # One extension: the object it names, the DER of its value, and whether
    # a reader that does not know it must refuse the certificate.
    class Extension
      attr_reader :oid

      # The value is DER, or an ASN.1 value written as DER.
      def initialize(oid, value, critical = false, shown = nil)
        found = OpenSSL.__object__(:find, oid.to_s)
        @oid = found ? found[0] : oid.to_s
        value = value.to_der if value.respond_to?(:to_der)
        @der_value = value.b
        @critical = critical
        @shown = shown
      end

      def critical?
        @critical
      end

      def value
        @shown || Extension.__shown__(@oid, @der_value) || Extension.__printable__(@der_value)
      end

      # ASN1_STRING_print: the bytes as text, with each one outside printable
      # ASCII, other than a line ending, written as a dot.
      def self.__printable__(bytes)
        bytes.b.bytes.map { |byte| (byte < 0x20 && byte != 0x0a && byte != 0x0d) || byte > 0x7e ? "." : byte.chr }.join
      end

      def self.__hex__(bytes)
        bytes.unpack1("H*").scan(/../).map(&:upcase).join(":")
      end

      # The text OpenSSL writes for the DER of the extensions it knows, or nil
      # for one it would show as bytes.
      def self.__shown__(oid, der)
        top = OpenSSL::ASN1.__read_der__(der)
        return nil if top.nil?
        tag, content = top
        case oid
        when "crlNumber"
          tag == OpenSSL::ASN1::INTEGER ? OpenSSL::ASN1.__signed_value__(content).to_s : nil
        when "CRLReason"
          tag == OpenSSL::ASN1::ENUMERATED ? OpenSSL::X509::CRL_REASONS[OpenSSL::ASN1.__signed_value__(content)] : nil
        when "basicConstraints"
          parts = OpenSSL::ASN1.__read_all__(content) || []
          authority = parts.any? { |held_tag, held| held_tag == OpenSSL::ASN1::BOOLEAN && held != "\x00".b }
          length = parts.find { |held_tag, _| held_tag == OpenSSL::ASN1::INTEGER }
          shown = "CA:#{authority ? "TRUE" : "FALSE"}"
          length ? "#{shown}, pathlen:#{OpenSSL::ASN1.__integer_value__(length[1])}" : shown
        when "keyUsage"
          return nil unless tag == OpenSSL::ASN1::BIT_STRING
          bytes = content.byteslice(1..).bytes
          names = ExtensionFactory::KEY_USAGE_BITS.values.filter_map do |bit, name|
            name if bit / 8 < bytes.size && (bytes[bit / 8] & (0x80 >> (bit % 8))) != 0
          end
          names.join(", ")
        when "extendedKeyUsage"
          (OpenSSL::ASN1.__read_all__(content) || []).map do |_, held|
            dotted = OpenSSL::ASN1.__oid_text__(held)
            found = OpenSSL.__object__(:find, dotted)
            found ? found[1] : dotted
          end.join(", ")
        when "subjectKeyIdentifier"
          __hex__(content)
        when "authorityKeyIdentifier"
          (OpenSSL::ASN1.__read_all__(content) || []).filter_map do |held_tag, held|
            held_tag == 0x80 ? __hex__(held) : nil
          end.join(", ")
        when "subjectAltName", "issuerAltName"
          (OpenSSL::ASN1.__read_all__(content) || []).map { |name| __general_name__(*name) }.join(", ")
        when "crlDistributionPoints"
          points = (OpenSSL::ASN1.__read_all__(content) || []).map do |_, point|
            parts = OpenSSL::ASN1.__read_all__(point) || []
            return nil unless parts.size == 1 && parts[0][0] == 0xa0
            named = OpenSSL::ASN1.__read_all__(parts[0][1]) || []
            return nil unless named.size == 1 && named[0][0] == 0xa0
            names = OpenSSL::ASN1.__read_all__(named[0][1]) || []
            "Full Name:\n" + names.map { |name| "  #{__general_name__(*name)}\n" }.join
          end
          points.join("\n")
        when "authorityInfoAccess"
          (OpenSSL::ASN1.__read_all__(content) || []).map do |_, description|
            method, location = OpenSSL::ASN1.__read_all__(description)
            dotted = OpenSSL::ASN1.__oid_text__(method[1])
            found = OpenSSL.__object__(:find, dotted)
            "#{found ? found[1] : dotted} - #{__general_name__(*location)}"
          end.join("\n")
        when "certificatePolicies"
          lines = []
          (OpenSSL::ASN1.__read_all__(content) || []).each do |_, information|
            policy, qualifiers = OpenSSL::ASN1.__read_all__(information)
            dotted = OpenSSL::ASN1.__oid_text__(policy[1])
            found = OpenSSL.__object__(:find, dotted)
            lines << "Policy: #{found ? found[1] : dotted}"
            next if qualifiers.nil?
            (OpenSSL::ASN1.__read_all__(qualifiers[1]) || []).each do |_, qualifier|
              kind, detail = OpenSSL::ASN1.__read_all__(qualifier)
              case OpenSSL::ASN1.__oid_text__(kind[1])
              when "1.3.6.1.5.5.7.2.1" then lines << "  CPS: #{detail[1]}"
              when "1.3.6.1.5.5.7.2.2"
                lines << "  User Notice:"
                notice = OpenSSL::ASN1.__read_all__(detail[1]) || []
                return nil unless notice.all? { |tag, _| tag != 0x30 }
                # OpenSSL writes the text as C characters, which end at the
                # first zero byte, as the high byte of a BMPString does.
                notice.each { |_, text| lines << "    Explicit Text: #{text.b.split("\x00".b, 2).first}" }
              else return nil
              end
            end
          end
          lines.join("\n")
        end
      end

      # A GeneralName the way OpenSSL writes it.
      def self.__general_name__(tag, held)
        case tag
        when 0x81 then "email:#{held}"
        when 0x82 then "DNS:#{held}"
        when 0x86 then "URI:#{held}"
        when 0x87
          address = held.bytesize == 4 ? held.bytes.join(".") : held.unpack("n*").map { |group| group.to_s(16).upcase }.join(":")
          "IP Address:#{address}"
        when 0xa4
          name = OpenSSL::ASN1.__read_der__(held)
          "DirName:#{Name.new(OpenSSL::ASN1.__encoded__(0x30, name[1]))}"
        else "othername:<unsupported>"
        end
      end

      def value_der
        @der_value
      end

      def to_der
        named = OpenSSL.__object__(:find, @oid)[2]
        parts = [OpenSSL::ASN1.__object__(named)]
        parts << OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::BOOLEAN, "\xFF".b) if @critical
        parts << OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::OCTET_STRING, @der_value)
        OpenSSL::ASN1.__sequence__(*parts)
      end

      def to_s
        "#{@oid} = #{"critical, " if @critical}#{value.gsub("\n", ", ")}"
      end

      def to_a
        [@oid, value, @critical]
      end
    end

    # Builds extensions from the text OpenSSL's configuration files write
    # them in.
    class ExtensionFactory
      attr_accessor :subject_certificate, :issuer_certificate, :subject_request, :crl, :config

      KEY_USAGE_BITS = {
        "digitalSignature" => [0, "Digital Signature"],
        "nonRepudiation" => [1, "Non Repudiation"],
        "keyEncipherment" => [2, "Key Encipherment"],
        "dataEncipherment" => [3, "Data Encipherment"],
        "keyAgreement" => [4, "Key Agreement"],
        "keyCertSign" => [5, "Certificate Sign"],
        "cRLSign" => [6, "CRL Sign"],
        "encipherOnly" => [7, "Encipher Only"],
        "decipherOnly" => [8, "Decipher Only"]
      }.freeze

      def initialize(issuer = nil, subject = nil, request = nil, crl = nil)
        @issuer_certificate = issuer
        @subject_certificate = subject
        @subject_request = request
        @crl = crl
      end

      def create_extension(oid, value, critical = false)
        found = OpenSSL.__object__(:find, oid.to_s)
        named = found && found[1]
        der, shown = case named
                     when "X509v3 Basic Constraints" then basic_constraints(value)
                     when "X509v3 Key Usage" then key_usage(value)
                     when "X509v3 Subject Key Identifier" then subject_key_identifier(value)
                     when "X509v3 Authority Key Identifier" then authority_key_identifier(value)
                     when "X509v3 Extended Key Usage" then extended_key_usage(value)
                     when "Netscape Comment" then [OpenSSL::ASN1.__encoded__(22, value.to_s.b), value.to_s]
                     when "X509v3 CRL Distribution Points" then crl_distribution_points(value)
                     end
        if der.nil?
          raise ExtensionError, "#{oid} = #{value}: error in extension (name=#{oid}, value=#{value})"
        end
        Extension.new(found[0], der, critical, shown)
      end

      private

      def basic_constraints(value)
        parts = value.split(",").map(&:strip)
        authority = parts.include?("CA:TRUE")
        length = parts.find { |part| part.start_with?("pathlen:") }&.split(":", 2)&.last
        body = []
        body << OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::BOOLEAN, "\xFF".b) if authority
        body << OpenSSL::ASN1.__integer__(length.to_i) if length
        [OpenSSL::ASN1.__sequence__(*body), value]
      end

      # A BIT STRING of the uses named, trimmed to the last bit set.
      def key_usage(value)
        names = value.split(",").map(&:strip)
        return nil unless names.all? { |name| KEY_USAGE_BITS.key?(name) }
        bits = names.map { |name| KEY_USAGE_BITS[name][0] }
        width = bits.max + 1
        bytes = Array.new((width + 7) / 8, 0)
        bits.each { |bit| bytes[bit / 8] |= 0x80 >> (bit % 8) }
        unused = bytes.size * 8 - width
        shown = names.map { |name| KEY_USAGE_BITS[name][1] }.join(", ")
        [OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::BIT_STRING, unused.chr + bytes.pack("C*")), shown]
      end

      def subject_key_identifier(value)
        return nil unless value == "hash" && @subject_certificate&.public_key
        identifier = OpenSSL::X509.__key_identifier__(@subject_certificate.public_key)
        [OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::OCTET_STRING, identifier), hex(identifier)]
      end

      def authority_key_identifier(value)
        return nil unless value.start_with?("keyid") && @issuer_certificate&.public_key
        identifier = OpenSSL::X509.__key_identifier__(@issuer_certificate.public_key)
        [OpenSSL::ASN1.__sequence__(OpenSSL::ASN1.__encoded__(0x80, identifier)), hex(identifier)]
      end

      # A SEQUENCE of the purposes named, each by its object identifier.
      def extended_key_usage(value)
        found = value.split(",").map { |name| OpenSSL.__object__(:find, name.strip) }
        return nil if found.empty? || found.any?(&:nil?)
        purposes = found.map { |held| OpenSSL::ASN1.__object__(held[2]) }
        [OpenSSL::ASN1.__sequence__(*purposes), found.map { |held| held[1] }.join(", ")]
      end

      # The tags of the general names a distribution point may be given by.
      GENERAL_NAME_TAGS = { "email" => 0x81, "DNS" => 0x82, "URI" => 0x86 }.freeze

      # A distribution point for each full name given, written inline as
      # `URI:...` or as `@section` naming a section of the config.
      def crl_distribution_points(value)
        entries = if value.start_with?("@")
                    @config ? @config[value.delete_prefix("@")].map { |name, held| [name.split(".").first, held] } : []
                  else
                    value.split(",").map { |held| held.strip.split(":", 2) }
                  end
        return nil if entries.empty? || entries.any? { |kind, held| held.nil? || !GENERAL_NAME_TAGS.key?(kind) }
        points = entries.map do |kind, held|
          name = OpenSSL::ASN1.__encoded__(GENERAL_NAME_TAGS[kind], held.b)
          OpenSSL::ASN1.__sequence__(OpenSSL::ASN1.__encoded__(0xa0, OpenSSL::ASN1.__encoded__(0xa0, name)))
        end
        der = OpenSSL::ASN1.__sequence__(*points)
        [der, nil]
      end

      def hex(bytes)
        bytes.unpack1("H*").scan(/../).map(&:upcase).join(":")
      end
    end

    # The identifier a key is known by: the SHA-1 of its public key bits.
    def self.__key_identifier__(key)
      OpenSSL::Digest.digest("SHA1", key.__public_key_bits__).b
    end

    SIGNATURE_ALGORITHMS = {
      "SHA1" => ["sha1WithRSAEncryption", "1.2.840.113549.1.1.5"],
      "SHA256" => ["sha256WithRSAEncryption", "1.2.840.113549.1.1.11"],
      "SHA384" => ["sha384WithRSAEncryption", "1.2.840.113549.1.1.12"],
      "SHA512" => ["sha512WithRSAEncryption", "1.2.840.113549.1.1.13"]
    }.freeze

    class Certificate
      attr_reader :version, :subject, :issuer, :extensions

      # An empty certificate, or the one PEM or DER text holds.
      def initialize(text = nil)
        @version = 0
        @serial = 0
        @subject = Name.new
        @issuer = Name.new
        @extensions = []
        return if text.nil?
        text = text.read if text.respond_to?(:read)
        text = OpenSSL::KDF.coerced(text)
        der = text.b
        if der.include?("-----BEGIN CERTIFICATE-----")
          matched = der.match(/-----BEGIN CERTIFICATE-----(.*?)-----END CERTIFICATE-----/m)
          der = matched[1].unpack1("m")
        end
        __read__(der)
      rescue ArgumentError, NoMethodError, TypeError
        raise CertificateError, "PEM_read_bio_X509: no start line"
      end

      # The fields of a Certificate SEQUENCE: what is signed, the
      # algorithm, and the signature.
      def __read__(der)
        top = OpenSSL::ASN1.__read_der__(der)
        raise CertificateError, "PEM_read_bio_X509: no start line" unless top && top[0] == 0x30
        (_, tbs), (_, algorithm), (_, signature) = OpenSSL::ASN1.__read_all__(top[1])
        @der = OpenSSL::ASN1.__encoded__(0x30, top[1])
        @tbs = OpenSSL::ASN1.__encoded__(0x30, tbs)
        fields = OpenSSL::ASN1.__read_all__(tbs)
        if fields.first[0] == 0xa0
          @version = OpenSSL::ASN1.__integer_value__(OpenSSL::ASN1.__read_der__(fields.shift[1])[1])
        end
        @serial = OpenSSL::ASN1.__integer_value__(fields.shift[1])
        fields.shift
        @issuer = Name.new(OpenSSL::ASN1.__encoded__(0x30, fields.shift[1]))
        @not_before, @not_after = OpenSSL::ASN1.__read_all__(fields.shift[1]).map do |tag, content|
          OpenSSL::ASN1.__time_value__(tag, content)
        end
        @subject = Name.new(OpenSSL::ASN1.__encoded__(0x30, fields.shift[1]))
        @public_key = OpenSSL::PKey.read(OpenSSL::ASN1.__encoded__(0x30, fields.shift[1]))
        fields.each do |tag, content|
          next unless tag == 0xa3
          listed = OpenSSL::ASN1.__read_der__(content)[1]
          @extensions = OpenSSL::ASN1.__read_all__(listed).map { |_, extension| Certificate.__extension__(extension) }
        end
        oid = OpenSSL::ASN1.__oid_text__(OpenSSL::ASN1.__read_all__(algorithm).first[1])
        @digest_name, (@signature_algorithm, @signature_oid) = SIGNATURE_ALGORITHMS.find { |_, (_, held)| held == oid }
        # A signature made some other way, such as with ECDSA, is still named
        # the way OpenSSL names it.
        if @signature_algorithm.nil?
          found = OpenSSL.__object__(:find, oid)
          @signature_algorithm = found ? found[1] : oid
          @signature_oid = oid
        end
        @signature = signature.byteslice(1..)
      end
      private :__read__

      # The certificate as OpenSSL's X509_print writes it.
      def to_text
        lines = ["Certificate:", "    Data:"]
        lines << "        Version: #{@version + 1} (0x#{@version.to_s(16)})"
        number = serial.to_i
        if number.bit_length <= 63
          lines << "        Serial Number: #{number} (0x#{number.to_s(16)})"
        else
          lines << "        Serial Number:"
          lines << "            #{OpenSSL::ASN1.__integer_bytes__(number).unpack1("H*").scan(/../).join(":")}"
        end
        lines << "        Signature Algorithm: #{signature_algorithm}"
        lines << "        Issuer: #{Certificate.__printed_name__(@issuer)}"
        lines << "        Validity"
        lines << "            Not Before: #{Certificate.__printed_time__(not_before)}"
        lines << "            Not After : #{Certificate.__printed_time__(not_after)}"
        lines << "        Subject: #{Certificate.__printed_name__(@subject)}"
        lines << "        Subject Public Key Info:"
        lines.concat(public_key.__printed_lines__)
        unless @extensions.empty?
          lines << "        X509v3 extensions:"
          @extensions.each do |extension|
            found = OpenSSL.__object__(:find, extension.oid)
            named = found ? found[1] : extension.oid
            lines << "            #{named}: #{"critical" if extension.critical?}"
            extension.value.split("\n", -1).each { |line| lines << (line.empty? ? "" : "                #{line}") }
          end
        end
        lines << "    Signature Algorithm: #{signature_algorithm}"
        lines << "    Signature Value:"
        signature_bytes = @signature.to_s.b.byteslice(0..) || "".b
        signature_bytes.unpack1("H*").scan(/../).each_slice(18).with_index do |row, at|
          last = at == (signature_bytes.bytesize - 1) / 18
          lines << "        #{row.join(":")}#{":" unless last}"
        end
        lines.join("\n") + "\n"
      end

      # X509_NAME_print: the one-line form, with each slash that opens a
      # field named by one or two capitals written as a comma instead.
      def self.__printed_name__(name)
        line = name.to_s
        return "" if line.empty?
        upper = ->(at) { (line[at] || "").match?(/\A[A-Z]\z/) }
        pieces = []
        start = 1
        at = 1
        loop do
          ending = at >= line.length
          opens_a_field = line[at] == "/" && upper.(at + 1) &&
                          (line[at + 2] == "=" || (upper.(at + 2) && line[at + 3] == "="))
          if ending || opens_a_field
            pieces << line[start...at]
            start = at + 1
          end
          break if ending
          at += 1
        end
        pieces.join(", ")
      end

      def self.__printed_time__(time)
        time.utc.strftime("%b %e %H:%M:%S %Y GMT")
      end

      def self.__extension__(der)
        parts = OpenSSL::ASN1.__read_all__(der)
        oid = OpenSSL::ASN1.__oid_text__(parts.first[1])
        critical = parts.size == 3 && parts[1][1] == "\xFF".b
        found = OpenSSL.__object__(:find, oid)
        Extension.new(found ? found[0] : oid, parts.last[1], critical)
      end

      def to_pem
        "-----BEGIN CERTIFICATE-----\n#{[to_der].pack("m64")}-----END CERTIFICATE-----\n"
      end
      alias to_s to_pem

      # Two certificates are equal when they encode alike. One never signed
      # has no encoding to compare, and OpenSSL counts any two such as equal.
      def ==(other)
        return false unless other.is_a?(Certificate)
        return other.__unsigned__? if __unsigned__?
        !other.__unsigned__? && to_der == other.to_der
      end

      def __unsigned__?
        @signature.nil? && @der.nil?
      end

      def version=(value)
        @der = nil
        @version = value
      end

      def serial
        OpenSSL::BN.new(@serial)
      end

      def serial=(value)
        @der = nil
        @serial = value.to_i
      end

      def subject=(name)
        @der = nil
        @subject = name
      end

      def issuer=(name)
        @der = nil
        @issuer = name
      end

      def public_key
        raise CertificateError, "decode error" if @public_key.nil?
        @public_key
      end

      def signature_algorithm
        raise OpenSSL::OpenSSLError, "OBJ_obj2txt" if @signature.nil?
        @signature_algorithm
      end

      def public_key=(key)
        @der = nil
        @public_key = key.public_key
      end

      def not_before
        raise OpenSSL::ASN1::ASN1Error, "ASN1_TIME_to_tm" if @not_before.nil?
        @not_before
      end

      def not_before=(time)
        @der = nil
        @not_before = Time.at(time.to_i).utc
      end

      def not_after
        raise OpenSSL::ASN1::ASN1Error, "ASN1_TIME_to_tm" if @not_after.nil?
        @not_after
      end

      def not_after=(time)
        @der = nil
        @not_after = Time.at(time.to_i).utc
      end

      def add_extension(extension)
        @der = nil
        @extensions << extension
        extension
      end

      def extensions=(list)
        @der = nil
        @extensions = list.to_a
      end

      def sign(key, digest)
        @der = nil
        named = OpenSSL::Digest.canonical_name(digest.respond_to?(:name) ? digest.name : digest.to_s)
        @signature_algorithm, @signature_oid = SIGNATURE_ALGORITHMS.fetch(named)
        @digest_name = named
        @signature = key.sign(named, tbs_der)
        self
      end

      def verify(key)
        raise CertificateError, "unknown signature algorithm" if @signature.nil?
        key.verify(@digest_name, @signature, tbs_der)
      end

      def to_der
        return @der if @der
        raise CertificateError, "illegal zero content" if @signature.nil?
        OpenSSL::ASN1.__sequence__(
          tbs_der,
          algorithm_der,
          OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::BIT_STRING, "\x00".b + @signature)
        )
      end

      # Issued by itself: its subject names its issuer.
      def __self_issued__
        @subject.cmp(@issuer).zero?
      end

      def __valid_at__(time)
        return :not_yet if !@not_before.nil? && time < @not_before
        return :expired if !@not_after.nil? && time > @not_after
        :valid
      end

      private

      def algorithm_der
        OpenSSL::ASN1.__sequence__(OpenSSL::ASN1.__object__(@signature_oid), OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::NULL, ""))
      end

      # TBSCertificate: what the signature covers.
      def tbs_der
        return @tbs if @der
        parts = []
        parts << OpenSSL::ASN1.__encoded__(0xa0, OpenSSL::ASN1.__integer__(@version)) unless @version.zero?
        parts << OpenSSL::ASN1.__integer__(@serial)
        parts << algorithm_der
        parts << @issuer.to_der
        parts << OpenSSL::ASN1.__sequence__(OpenSSL::ASN1.__time__(not_before), OpenSSL::ASN1.__time__(not_after))
        parts << @subject.to_der
        parts << @public_key.public_to_der
        unless @extensions.empty?
          listed = OpenSSL::ASN1.__sequence__(*@extensions.map(&:to_der))
          parts << OpenSSL::ASN1.__encoded__(0xa3, listed)
        end
        OpenSSL::ASN1.__sequence__(*parts)
      end
    end

    # A certificate a CRL names as revoked: its serial, when it was revoked,
    # and the extensions that say why.
    class Revoked
      attr_reader :time, :extensions

      def initialize
        @serial = 0
        @time = nil
        @extensions = []
      end

      def serial
        OpenSSL::BN.new(@serial)
      end

      def serial=(value)
        @serial = value.to_i
      end

      def time=(value)
        @time = value
      end

      def extensions=(list)
        @extensions = list.to_a
      end

      def add_extension(extension)
        @extensions << extension
        extension
      end

      def to_der
        parts = [OpenSSL::ASN1.__integer__(@serial), OpenSSL::ASN1.__time__(@time)]
        parts << OpenSSL::ASN1.__sequence__(*@extensions.map(&:to_der)) unless @extensions.empty?
        OpenSSL::ASN1.__sequence__(*parts)
      end

      def ==(other)
        other.is_a?(Revoked) && other.to_der == to_der
      end
    end

    # A certificate revocation list: the certificates an issuer has revoked,
    # signed by that issuer.
    class CRL
      class CRLError < OpenSSL::OpenSSLError; end

      attr_reader :version, :issuer, :last_update, :next_update, :revoked, :extensions

      def initialize
        @version = 0
        @issuer = Name.new
        @last_update = nil
        @next_update = nil
        @revoked = []
        @extensions = []
        @signature = nil
      end

      def version=(value)
        @version = value.to_i
      end

      def issuer=(name)
        @issuer = name
      end

      def last_update=(time)
        @last_update = time
      end

      def next_update=(time)
        @next_update = time
      end

      def revoked=(list)
        @revoked = list.to_a
      end

      def add_revoked(revoked)
        @revoked << revoked
        revoked
      end

      def extensions=(list)
        @extensions = list.to_a
      end

      def add_extension(extension)
        @extensions << extension
        extension
      end

      def signature_algorithm
        @signature_algorithm || "NULL"
      end

      def sign(key, digest)
        named = OpenSSL::Digest.canonical_name(digest.respond_to?(:name) ? digest.name : digest.to_s)
        @signature_algorithm, @signature_oid = SIGNATURE_ALGORITHMS.fetch(named)
        @digest_name = named
        @signature = key.sign(named, __tbs_der__)
        self
      end

      def verify(key)
        return false if @signature.nil?
        key.verify(@digest_name, @signature, __tbs_der__)
      end

      def to_der
        raise CRLError, "illegal zero content" if @signature.nil?
        OpenSSL::ASN1.__sequence__(
          __tbs_der__,
          __algorithm_der__,
          OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::BIT_STRING, "\x00".b + @signature)
        )
      end

      def to_pem
        "-----BEGIN X509 CRL-----\n#{[to_der].pack("m64")}-----END X509 CRL-----\n"
      end
      alias to_s to_pem

      def ==(other)
        other.is_a?(CRL) && other.to_der == to_der
      end

      def __algorithm_der__
        OpenSSL::ASN1.__sequence__(OpenSSL::ASN1.__object__(@signature_oid), OpenSSL::ASN1.__encoded__(OpenSSL::ASN1::NULL, ""))
      end
      private :__algorithm_der__

      # TBSCertList: what the signature covers.
      def __tbs_der__
        parts = []
        parts << OpenSSL::ASN1.__integer__(@version) unless @version.zero?
        parts << __algorithm_der__
        parts << @issuer.to_der
        parts << OpenSSL::ASN1.__time__(@last_update)
        parts << OpenSSL::ASN1.__time__(@next_update) if @next_update
        parts << OpenSSL::ASN1.__sequence__(*@revoked.map(&:to_der)) unless @revoked.empty?
        unless @extensions.empty?
          parts << OpenSSL::ASN1.__encoded__(0xa0, OpenSSL::ASN1.__sequence__(*@extensions.map(&:to_der)))
        end
        OpenSSL::ASN1.__sequence__(*parts)
      end
      private :__tbs_der__
    end

    # The certificates trusted to anchor a chain, and the result of the last
    # verification made against them.
    # One verification of a certificate against a store: the chain it builds,
    # and where and why it stopped.
    class StoreContext
      attr_reader :chain, :error_depth, :current_cert
      attr_accessor :error, :time, :flags, :purpose, :trust
      attr_writer :__callback__, :__adjust__

      def initialize(store, certificate = nil, untrusted = nil)
        @store = store
        @certificate = certificate
        @untrusted = untrusted.to_a
        @error = V_OK
        @error_depth = 0
        @chain = nil
        @time = store.time
        @flags = store.flags.to_i
        @current_crl = nil
      end

      def error_string
        VERIFY_ERROR_STRINGS.fetch(@error, "certificate verify error")
      end

      attr_reader :current_crl

      def cleanup
        self
      end

      # Build the chain up to a root the store trusts, then check each
      # certificate's signature and validity from the root down. Every
      # failure, and every certificate that passes, goes to the callback,
      # which may accept the failure.
      def verify
        trusted = @store.__trusted__
        @chain = [@certificate]
        failure = __build__(trusted)
        if failure
          return false unless __step__(false, *failure)
        end
        (1...@chain.size).each do |depth|
          next if StoreContext.__may_issue__(@chain[depth])
          return false unless __step__(false, V_ERR_INVALID_CA, depth)
        end
        return false unless __check_revocation__
        __check_down__
      end

      # With CRL checking asked for, each certificate's issuer must have a
      # CRL in the store that is current, that the issuer signed, and that
      # does not name the certificate: the leaf alone, or the whole chain
      # with V_FLAG_CRL_CHECK_ALL.
      def __check_revocation__
        return true if (@flags & V_FLAG_CRL_CHECK).zero?
        last = (@flags & V_FLAG_CRL_CHECK_ALL).zero? ? 0 : @chain.size - 1
        now = @time || Time.now
        (0..last).each do |depth|
          held = @chain[depth]
          issuer = @chain[depth + 1] || held
          crl = @store.__crls__.find { |list| list.issuer.cmp(held.issuer).zero? }
          if crl.nil?
            return false unless __step__(false, V_ERR_UNABLE_TO_GET_CRL, depth)
            next
          end
          @current_crl = crl
          if crl.last_update && now < crl.last_update
            return false unless __step__(false, V_ERR_CRL_NOT_YET_VALID, depth)
          elsif crl.next_update && now > crl.next_update
            return false unless __step__(false, V_ERR_CRL_HAS_EXPIRED, depth)
          end
          unless __crl_signed_by__(crl, issuer)
            return false unless __step__(false, V_ERR_CRL_SIGNATURE_FAILURE, depth)
          end
          if crl.revoked.any? { |entry| entry.serial == held.serial }
            return false unless __step__(false, V_ERR_CERT_REVOKED, depth)
          end
        end
        true
      end

      def __crl_signed_by__(crl, issuer)
        crl.verify(issuer.public_key)
      rescue StandardError
        false
      end

      # Whether a certificate may sign others, as OpenSSL's check_ca decides:
      # a key usage, when there is one, must allow signing certificates, then
      # basic constraints decide, and without them a self-signed version 1
      # certificate or one with a key usage may.
      def self.__may_issue__(certificate)
        named = certificate.extensions.to_h { |extension| [extension.oid, extension.value] }
        usage = named["keyUsage"]
        return false if usage && !usage.split(", ").include?("Certificate Sign")
        constraints = named["basicConstraints"]
        return constraints.start_with?("CA:TRUE") if constraints
        return true if certificate.version.zero? && certificate.__self_issued__
        !usage.nil?
      end

      # The chain from the certificate up, and the failure that stopped it
      # short of a trusted root, if one did.
      def __build__(trusted)
        current = @certificate
        loop do
          if current.__self_issued__
            return nil if trusted.include?(current)
            code = @chain.size == 1 ? V_ERR_DEPTH_ZERO_SELF_SIGNED_CERT : V_ERR_SELF_SIGNED_CERT_IN_CHAIN
            return [code, @chain.size - 1]
          end
          issuer = __issuer_of__(current, trusted) || __issuer_of__(current, @untrusted)
          if issuer.nil?
            code = trusted.include?(current) ? V_ERR_UNABLE_TO_GET_ISSUER_CERT : V_ERR_UNABLE_TO_GET_ISSUER_CERT_LOCALLY
            return [code, @chain.size - 1]
          end
          @chain << issuer
          current = issuer
        end
      end

      # A certificate named as the issuer, preferring one whose key the
      # signature verifies with.
      def __issuer_of__(certificate, candidates)
        named = candidates.select do |held|
          !held.equal?(certificate) && held.subject.cmp(certificate.issuer).zero?
        end
        named.find { |held| __signed_by__(certificate, held) } || named.first
      end

      def __signed_by__(certificate, issuer)
        certificate.verify(issuer.public_key)
      rescue StandardError
        false
      end

      def __check_down__
        depth = @chain.size - 1
        top = @chain[depth]
        issuer = top
        if top.__self_issued__
          held = top
        elsif depth.zero?
          return false unless __step__(false, V_ERR_UNABLE_TO_VERIFY_LEAF_SIGNATURE, 0)
          held = top
        else
          depth -= 1
          held = @chain[depth]
        end
        now = @time || Time.now
        loop do
          if !held.equal?(issuer) && !__signed_by__(held, issuer)
            return false unless __step__(false, V_ERR_CERT_SIGNATURE_FAILURE, depth)
          end
          case held.__valid_at__(now)
          when :not_yet then return false unless __step__(false, V_ERR_CERT_NOT_YET_VALID, depth)
          when :expired then return false unless __step__(false, V_ERR_CERT_HAS_EXPIRED, depth)
          end
          return false unless __step__(true, @error, depth)
          return true if depth.zero?
          issuer = held
          depth -= 1
          held = @chain[depth]
        end
      end

      # Hand one step to the callback the way Ruby's OpenSSL does: only an
      # answer of true accepts it, and an exception counts as refusing.
      def __step__(ok, code, depth)
        @error = code
        @error_depth = depth
        @current_cert = @chain[depth]
        ok = @__adjust__.call(ok, self) if @__adjust__
        return ok if @__callback__.nil?
        answer = begin
          @__callback__.call(ok, self)
        rescue StandardError
          warn "exception in verify_callback is ignored", uplevel: 0
          false
        end
        if answer == true
          @error = V_OK
          return true
        end
        @error = V_ERR_CERT_REJECTED if @error == V_OK
        false
      end
    end

    DEFAULT_CERT_FILE_ENV = "SSL_CERT_FILE"
    DEFAULT_CERT_DIR_ENV = "SSL_CERT_DIR"
    DEFAULT_CERT_FILE = "/etc/ssl/cert.pem"
    DEFAULT_CERT_DIR = "/etc/ssl/certs"

    class Store
      attr_reader :error, :chain
      attr_accessor :time, :flags, :purpose, :trust, :verify_callback

      def initialize
        @trusted = []
        @crls = []
        @error = 0
        @chain = nil
      end

      def add_cert(certificate)
        @trusted << certificate
        self
      end

      def add_crl(crl)
        @crls << crl
        self
      end

      def __crls__
        @crls
      end

      def __trusted__
        __load_default_paths__
        @trusted
      end

      # Trust the certificates of the file and directory the environment
      # names, or of the system's own, read when they are first needed.
      def set_default_paths
        @default_paths = true
        self
      end

      def __load_default_paths__
        return unless @default_paths
        @default_paths = false
        file = ENV[DEFAULT_CERT_FILE_ENV] || DEFAULT_CERT_FILE
        add_file(file) if File.file?(file)
        directory = ENV[DEFAULT_CERT_DIR_ENV] || DEFAULT_CERT_DIR
        directory.split(File::PATH_SEPARATOR).each do |held|
          next unless File.directory?(held)
          # A directory is read the way OpenSSL looks in one: by the files
          # named for the hash of a subject.
          Dir.children(held).grep(/\A\h{8}\.\d+\z/).sort.each do |name|
            add_file(File.join(held, name)) if File.file?(File.join(held, name))
          end
        end
      rescue StoreError
        nil
      end
      private :__load_default_paths__

      # Trust each certificate a PEM file holds.
      def add_file(path)
        File.read(path).scan(/-----BEGIN CERTIFICATE-----.*?-----END CERTIFICATE-----/m) do |pem|
          add_cert(Certificate.new(pem))
        end
        self
      rescue SystemCallError
        raise StoreError, "X509_LOOKUP_load_file: system lib"
      end

      # Trust the certificates of each file in a directory.
      def add_path(path)
        Dir.children(path).sort.each do |name|
          held = File.join(path, name)
          add_file(held) if File.file?(held)
        end
        self
      end

      def error_string
        VERIFY_ERROR_STRINGS.fetch(@error, "certificate verify error")
      end

      # Verify a certificate the way X509_verify_cert does, handing each
      # step to the block or to the store's verify_callback.
      def verify(certificate, untrusted = nil, &block)
        context = StoreContext.new(self, certificate, untrusted)
        context.__callback__ = block || @verify_callback
        trusted = context.verify
        @error = context.error
        @chain = context.chain
        trusted
      end
    end
  end
end

require "openssl/buffering"
require "socket"
require "ipaddr"

module OpenSSL
  module SSL
    OP_ALL = 0x80000850
    OP_DONT_INSERT_EMPTY_FRAGMENTS = 0x800
    OP_NO_COMPRESSION = 0x20000
    OP_NO_TICKET = 0x4000
    OP_NO_SSLv2 = 0
    OP_NO_SSLv3 = 0x2000000
    OP_NO_TLSv1 = 0x4000000
    OP_NO_TLSv1_1 = 0x10000000
    OP_NO_TLSv1_2 = 0x8000000
    OP_NO_TLSv1_3 = 0x20000000

    SSL2_VERSION = 0x0002
    SSL3_VERSION = 0x0300
    TLS1_VERSION = 0x0301
    TLS1_1_VERSION = 0x0302
    TLS1_2_VERSION = 0x0303
    TLS1_3_VERSION = 0x0304

    # The one cipher suite the connection speaks, as SSLSocket#cipher and
    # SSLContext#ciphers describe it.
    CIPHER = ["TLS_AES_128_GCM_SHA256", "TLSv1.3", 128, 128].freeze

    # The settings a TLS connection is made under.
    class SSLContext
      SESSION_CACHE_OFF = 0
      SESSION_CACHE_CLIENT = 1
      SESSION_CACHE_SERVER = 2
      SESSION_CACHE_BOTH = 3
      SESSION_CACHE_NO_AUTO_CLEAR = 0x80
      SESSION_CACHE_NO_INTERNAL_LOOKUP = 0x100
      SESSION_CACHE_NO_INTERNAL_STORE = 0x200
      SESSION_CACHE_NO_INTERNAL = 0x300

      DEFAULT_PARAMS = {
        verify_mode: OpenSSL::SSL::VERIFY_PEER,
        verify_hostname: true,
        options: (OpenSSL::SSL::OP_ALL & ~OpenSSL::SSL::OP_DONT_INSERT_EMPTY_FRAGMENTS) | OpenSSL::SSL::OP_NO_COMPRESSION
      }.freeze

      DEFAULT_CERT_STORE = OpenSSL::X509::Store.new
      DEFAULT_CERT_STORE.set_default_paths

      attr_accessor :cert, :key, :extra_chain_cert, :ca_file, :ca_path, :cert_store, :client_ca,
                    :verify_mode, :verify_hostname, :verify_depth, :verify_callback, :timeout,
                    :ssl_timeout, :options, :session_id_context, :session_cache_mode, :session_cache_size,
                    :servername_cb, :client_cert_cb, :alpn_protocols, :alpn_select_cb,
                    :npn_protocols, :npn_select_cb, :session_new_cb, :session_get_cb,
                    :session_remove_cb, :renegotiation_cb, :keylog_cb, :tmp_dh_callback, :security_level
      attr_writer :ciphersuites, :ecdh_curves, :groups, :sigalgs, :client_sigalgs, :tmp_dh,
                  :min_version, :max_version

      # The newest protocol version the context allows, which OpenSSL has no
      # reader for. Zero or nil sets no limit.
      def __max_version__ = @max_version

      def initialize(version = nil)
        @options = 0x120000
        @session_cache_mode = 2
        @security_level = 1
        self.ssl_version = version if version
        self.verify_mode = OpenSSL::SSL::VERIFY_NONE
        self.verify_hostname = false
      end

      def set_params(params = {})
        params = DEFAULT_PARAMS.merge(params)
        self.options |= params.delete(:options)
        params.each { |name, value| __send__("#{name}=", value) }
        if verify_mode != OpenSSL::SSL::VERIFY_NONE && !ca_file && !ca_path && !cert_store
          self.cert_store = DEFAULT_CERT_STORE
        end
        params
      end

      METHODS_MAP = {
        SSLv23: 0,
        SSLv2: OpenSSL::SSL::SSL2_VERSION,
        SSLv3: OpenSSL::SSL::SSL3_VERSION,
        TLSv1: OpenSSL::SSL::TLS1_VERSION,
        TLSv1_1: OpenSSL::SSL::TLS1_1_VERSION,
        TLSv1_2: OpenSSL::SSL::TLS1_2_VERSION
      }.freeze
      private_constant :METHODS_MAP

      def ssl_version=(method)
        method = method.to_s if method.is_a?(Symbol)
        method = $` if /(_client|_server)\z/ =~ method
        version = METHODS_MAP[method.intern] or raise ArgumentError, "unknown SSL method `%s'" % method
        self.min_version = self.max_version = version
      end

      def ciphers
        [CIPHER.dup]
      end

      # The words an OpenSSL cipher string names groups of ciphers by.
      CIPHER_KEYWORDS = %w[
        DEFAULT COMPLEMENTOFDEFAULT ALL COMPLEMENTOFALL HIGH MEDIUM LOW eNULL NULL aNULL
        kRSA aRSA RSA kDHE kEDH DH DHE EDH ADH kEECDH kECDHE ECDH ECDHE EECDH AECDH
        aDSS DSS aECDSA ECDSA AES AES128 AES256 AESGCM AESCCM AESCCM8 CHACHA20 CAMELLIA
        CAMELLIA128 CAMELLIA256 ARIA ARIA128 ARIA256 SHA SHA1 SHA256 SHA384 TLSv1.2 TLSv1.0
        TLSv1 SSLv3 PSK kPSK aPSK kECDHEPSK kDHEPSK kRSAPSK
      ].freeze

      # TLS 1.2 and older ciphers, which TLS 1.3 connections do not use. As
      # under OpenSSL, a list that leaves none of them is refused.
      def ciphers=(list)
        text = list.is_a?(Array) ? list.map { |held| held.is_a?(Array) ? held.first : held }.join(":") : list
        unless text.nil? || SSLContext.__matches_a_cipher__(text.to_s)
          raise SSLError, "SSL_CTX_set_cipher_list: no cipher match"
        end
        @ciphers = list
      end

      def self.__matches_a_cipher__(text)
        items = text.split(/[:, ]+/).reject(&:empty?)
        return false if items.any? { |item| %w[!ALL -ALL].include?(item) }
        items.any? do |item|
          next false if item.start_with?("!", "-", "@")
          item.delete_prefix("+").split("+").all? do |part|
            CIPHER_KEYWORDS.include?(part) || part.match?(/\A[A-Z0-9]+(-[A-Z0-9]+)+\z/)
          end
        end
      end

      def add_certificate(certificate, key, extra = nil)
        @cert = certificate
        @key = key
        @extra_chain_cert = extra
        self
      end

      # Settle the context for use, after which it can no longer change.
      # Only the first call does anything, and it answers true.
      def setup
        return nil if frozen?
        freeze
        true
      end

      # The certificates the context trusts: its store's, and those of its
      # CA file and directory.
      def __trusted_store__
        store = OpenSSL::X509::Store.new
        cert_store&.__trusted__&.each { |held| store.add_cert(held) }
        store.add_file(ca_file) if ca_file
        store.add_path(ca_path) if ca_path
        store
      end

      def session_add(_session) = false
      def session_remove(_session) = false
      def flush_sessions(_time = nil) = self
      def enable_fallback_scsv = nil
      def session_cache_stats = {}
    end

    module SocketForwarder
      def fileno = to_io.fileno
      def addr = to_io.addr
      def peeraddr = to_io.peeraddr
      def local_address = to_io.local_address
      def remote_address = to_io.remote_address
      def setsockopt(level, name, value) = to_io.setsockopt(level, name, value)
      def getsockopt(level, name) = to_io.getsockopt(level, name)
      def fcntl(*arguments) = to_io.fcntl(*arguments)
      def closed? = to_io.closed?
      def do_not_reverse_lookup=(flag)
        to_io.do_not_reverse_lookup = flag
      end
      def close_on_exec=(value)
        to_io.close_on_exec = value
      end
      def close_on_exec? = to_io.close_on_exec?
      def wait(*arguments) = to_io.wait(*arguments)
      def wait_readable(*arguments) = to_io.wait_readable(*arguments)
      def wait_writable(*arguments) = to_io.wait_writable(*arguments)
      def timeout = to_io.timeout
      def timeout=(value)
        to_io.timeout = value
      end
    end

    def verify_certificate_identity(cert, hostname)
      should_verify_common_name = true
      cert.extensions.each do |extension|
        next if extension.oid != "subjectAltName"
        names = OpenSSL::ASN1.__read_der__(extension.value_der)
        OpenSSL::ASN1.__read_all__(names[1]).each do |tag, value|
          case tag
          when 0x82
            should_verify_common_name = false
            return true if verify_hostname(hostname, value)
          when 0x87
            should_verify_common_name = false
            if value.size == 4 || value.size == 16
              begin
                return true if value == IPAddr.new(hostname).hton
              rescue IPAddr::InvalidAddressError
              end
            end
          end
        end
      end
      if should_verify_common_name
        cert.subject.to_a.each do |oid, value|
          return true if oid == "CN" && verify_hostname(hostname, value)
        end
      end
      false
    end
    module_function :verify_certificate_identity

    def verify_hostname(hostname, san)
      return false unless san.ascii_only?
      return false unless hostname.ascii_only?
      san_parts = san.downcase.split(".")
      return san == hostname if san_parts.size < 2
      host_parts = hostname.downcase.split(".")
      return false unless san_parts.size == host_parts.size
      return false unless verify_wildcard(host_parts.shift, san_parts.shift)
      san_parts.join(".") == host_parts.join(".")
    end
    module_function :verify_hostname

    def verify_wildcard(domain_component, san_component)
      parts = san_component.split("*", -1)
      return false if parts.size > 2
      return san_component == domain_component if parts.size == 1
      return false if domain_component.start_with?("xn--") && san_component != "*"
      parts[0].length + parts[1].length < domain_component.length &&
        domain_component.start_with?(parts[0]) &&
        domain_component.end_with?(parts[1])
    end
    module_function :verify_wildcard

    # The session an established connection runs in. Sessions are not
    # resumed, so each holds only what names it.
    class Session
      class SessionError < OpenSSL::OpenSSLError; end

      attr_reader :id, :time
      attr_accessor :timeout

      def initialize(socket)
        raise SessionError, "SSL session is not available" if socket.cipher.nil?
        @id = SecureRandom.bytes(32)
        @time = Time.now
        @timeout = 7200
      end

      def ==(other)
        other.is_a?(Session) && id == other.id
      end
    end

    # A TLS connection over an IO, which the handshake and each record are
    # read from and written to.
    class SSLSocket
      include Buffering
      include SocketForwarder

      attr_reader :hostname, :io, :context, :verify_result, :peer_cert_chain
      alias to_io io
      attr_accessor :sync_close
      attr_writer :session

      # A TLS record opens with its content type, version, and length.
      RECORD_HEADER_SIZE = 5

      # The alert a client sends when it refuses the server's chain, by the
      # verification error found.
      REFUSAL_ALERTS = { 2 => 48, 7 => 51, 9 => 45, 10 => 45, 18 => 48, 19 => 48, 20 => 48 }.freeze

      def self.open(remote_host, remote_port, local_host = nil, local_port = nil, context: nil)
        socket = ::TCPSocket.open(remote_host, remote_port, local_host, local_port)
        context.nil? ? new(socket) : new(socket, context)
      end

      def initialize(io, context = OpenSSL::SSL::SSLContext.new)
        unless io.is_a?(IO)
          raise TypeError, "wrong argument type #{io.class} (expected File)"
        end
        @io = io
        @context = context
        @sync_close = false
        @hostname = nil
        @verify_result = OpenSSL::X509::V_OK
        @peer_cert_chain = nil
        @connection = nil
        @state = nil
        @plaintext = "".b
        @peer_closed = false
        context.setup
        super()
      end

      def hostname=(name)
        @hostname = name
      end

      def connect
        handshake(:client, "SSL_connect", true)
        self
      end

      def connect_nonblock(exception: true)
        handshake(:client, "SSL_connect", false, exception)
      end

      def accept
        handshake(:server, "SSL_accept", true)
        self
      end

      def accept_nonblock(exception: true)
        handshake(:server, "SSL_accept", false, exception)
      end

      def sysread(length, buffer = nil)
        data = plaintext(length, true, "SSL_read")
        raise EOFError, "end of file reached" if data.nil?
        buffer ? buffer.replace(data) : data
      end

      def sysread_nonblock(length, buffer = nil, exception: true)
        data = plaintext(length, false, "SSL_read")
        if data == :wait_readable
          raise SSLErrorWaitReadable, "read would block" if exception
          return :wait_readable
        end
        if data.nil?
          raise EOFError, "end of file reached" if exception
          return nil
        end
        buffer ? buffer.replace(data) : data
      end

      def syswrite(data)
        data = data.to_s
        records = OpenSSL.__tls__(:write, @connection, data.b)
        __io__(:write, records)
        data.bytesize
      end

      def syswrite_nonblock(data, exception: true)
        syswrite(data)
      end

      def sysclose
        return if closed?
        stop
        io.close if sync_close
      end

      def stop
        return if @connection.nil?
        begin
          __io__(:write, OpenSSL.__tls__(:close, @connection)) unless @peer_closed
        rescue SystemCallError, IOError
        end
        OpenSSL.__tls__(:free, @connection)
        @connection = nil
      end
      private :stop

      def post_connection_check(hostname)
        raise SSLError, "Peer verification enabled, but no certificate received." if peer_cert.nil?
        unless OpenSSL::SSL.verify_certificate_identity(peer_cert, hostname)
          raise SSLError, "hostname \"#{hostname}\" does not match the server certificate"
        end
        true
      end

      def peer_cert
        @peer_cert_chain&.first
      end

      def cert
        @state && @mode == :server ? @context.cert : nil
      end

      def cipher
        @state == :established ? CIPHER.dup : nil
      end

      def ssl_version
        @state == :established ? "TLSv1.3" : nil
      end

      def state
        @state == :established ? "SSLOK " : "before SSL initialization"
      end

      def pending
        @plaintext.bytesize
      end

      def session
        @established_session ||= SSL::Session.new(self)
      rescue SSL::Session::SessionError
        nil
      end

      def session_reused? = false
      def alpn_protocol = nil
      def npn_protocol = nil
      def close_read = nil

      def close_write
        stop
      end

      private

      # Run the handshake as the client or the server until it ends. When
      # not blocking, answer :wait_readable once the IO has nothing more to
      # read, or raise SSLErrorWaitReadable when `exception` asks for it.
      def handshake(mode, label, blocking, exception = true)
        if mode == :server && @connection.nil? && refuse_older_protocol(label, blocking, exception) == :wait_readable
          return :wait_readable
        end
        if @connection.nil?
          @mode = mode
          if mode == :server && @context.servername_cb
            @hello ||= read_io(blocking)
            if @hello == :wait_readable
              @hello = nil
              raise SSLErrorWaitReadable, "read would block" if exception
              return :wait_readable
            end
            if @hello.nil?
              raise SSLError, "#{label} SYSCALL returned=5 errno=0 #{peer_text}state=error: unexpected eof while reading"
            end
            choose_context_for(OpenSSL.__tls__(:hello_name, @hello), label)
          end
          @connection = if mode == :client
                          OpenSSL.__tls__(:client, @hostname, *client_certificate)
                        else
                          key = @context.key
                          raise SSLError, "#{label} returned=1 errno=0 state=error: no shared cipher" if @context.cert.nil? || key.nil?
                          chain = [@context.cert, *@context.extra_chain_cert].map(&:to_der)
                          OpenSSL.__tls__(:server, chain, key.n.to_i, key.e.to_i, key.d.to_i, @context.verify_mode.to_i)
                        end
          @state = :handshake
          opening = OpenSSL.__tls__(:start, @connection)
          __io__(:write, opening) unless opening.empty?
          if @hello
            hello = @hello
            @hello = nil
            advance(OpenSSL.__tls__(:feed, @connection, hello), label)
          end
        end
        until @state == :established
          bytes = read_io(blocking)
          if bytes == :wait_readable
            raise SSLErrorWaitReadable, "read would block" if exception
            return :wait_readable
          end
          if bytes.nil?
            raise SSLError, "#{label} SYSCALL returned=5 errno=0 #{peer_text}state=error: unexpected eof while reading"
          end
          advance(OpenSSL.__tls__(:feed, @connection, bytes), label)
        end
        self
      end

      # A server hands the name the client asked for to the context's
      # servername_cb, and answers with the context that callback returns.
      def choose_context_for(name, label)
        return if name.nil?
        chosen = @context.servername_cb.call([self, name])
        return unless chosen.is_a?(SSLContext)
        chosen.setup
        @context = chosen
      rescue StandardError => error
        @state = :failed
        raise SSLError, "#{label} returned=1 errno=0 #{peer_text}state=error: #{error.message}"
      end

      # The chain and RSA key a client proves itself with when the server
      # asks, or nothing when the context holds none.
      def client_certificate
        key = @context.key
        return [] if @context.cert.nil? || !key.respond_to?(:n)
        chain = [@context.cert, *@context.extra_chain_cert].map(&:to_der)
        [chain, key.n.to_i, key.e.to_i, key.d.to_i]
      end

      # The protocol_version alert, in a record of its own.
      PROTOCOL_VERSION_ALERT = [0x15, 0x03, 0x03, 0x00, 0x02, 0x02, 0x46].pack("C*").freeze

      # A server held to a version older than TLS 1.3, the only one spoken
      # here, reads the client's hello and refuses it the way OpenSSL refuses
      # a client offering no version the server allows.
      def refuse_older_protocol(label, blocking, exception)
        highest = @context.__max_version__.to_i
        return if highest.zero? || highest >= OpenSSL::SSL::TLS1_3_VERSION
        hello = read_io(blocking)
        if hello == :wait_readable
          raise SSLErrorWaitReadable, "read would block" if exception
          return :wait_readable
        end
        __io__(:write, PROTOCOL_VERSION_ALERT) unless hello.nil?
        @state = :failed
        raise SSLError, "#{label} returned=1 errno=0 #{peer_text}state=error: no protocols available"
      end
      private :refuse_older_protocol

      # Call the IO's own method, past any module extending that one IO.
      # OpenSSL reads and writes the descriptor itself, so a wrapper a
      # program adds to the socket never sees the TLS records.
      def __io__(name, *arguments, **options)
        @io.class.instance_method(name).bind_call(@io, *arguments, **options)
      end
      private :__io__

      # The next TLS record from the IO: its five byte header, then as many
      # bytes as the header names. Reading no further than the record leaves
      # what follows it to the IO, so waiting on the IO still sees data the
      # peer sent after the handshake, as it does under OpenSSL.
      def read_io(blocking)
        @incoming ||= "".b
        loop do
          wanted = RECORD_HEADER_SIZE
          if @incoming.bytesize >= RECORD_HEADER_SIZE
            wanted += @incoming.byteslice(3, 2).unpack1("n")
            return @incoming.slice!(0, wanted) if @incoming.bytesize >= wanted
          end
          missing = wanted - @incoming.bytesize
          piece = blocking ? __io__(:readpartial, missing) : __io__(:read_nonblock, missing, exception: false)
          return piece if piece.nil? || piece == :wait_readable
          @incoming << piece.b
        end
      rescue EOFError
        nil
      end

      # Send what the connection answered, keep the data it read, and carry
      # on through a chain decision or a failure.
      def advance(progress, label)
        outgoing, read, closed, state, problem = progress
        __io__(:write, outgoing) unless outgoing.empty?
        @plaintext << read
        @peer_closed ||= closed
        @state = state
        raise SSLError, "#{label} returned=1 errno=0 #{peer_text}state=error: #{problem}" if state == :failed
        decide_chain(label) if state == :decide
      end

      # Verify the server's chain against the certificates the context
      # trusts, and its name against the hostname, then go on or refuse.
      def decide_chain(label)
        @peer_cert_chain = OpenSSL.__tls__(:peer_chain, @connection).map { |der| OpenSSL::X509::Certificate.new(der) }
        verification = OpenSSL::X509::StoreContext.new(@context.__trusted_store__, @peer_cert_chain.first, @peer_cert_chain.drop(1))
        verification.__callback__ = @context.verify_callback
        # A client checks the server's name at the last step, the one for the
        # server's own certificate, before the callback hears of it.
        if @context.verify_hostname && @hostname && @mode == :client
          verification.__adjust__ = lambda do |ok, held|
            next ok unless ok && held.error_depth.zero?
            next true if OpenSSL::SSL.verify_certificate_identity(held.current_cert, @hostname) == true
            held.error = OpenSSL::X509::V_ERR_HOSTNAME_MISMATCH
            false
          end
        end
        trusted = verification.verify
        @verify_result = verification.error
        if !trusted && (@context.verify_mode.to_i & OpenSSL::SSL::VERIFY_PEER) != 0
          alert = REFUSAL_ALERTS.fetch(@verify_result, 46)
          outgoing, = OpenSSL.__tls__(:decide, @connection, alert)
          __io__(:write, outgoing) unless outgoing.empty?
          @state = :failed
          reason = OpenSSL::X509::VERIFY_ERROR_STRINGS.fetch(@verify_result, "certificate verify failed")
          raise SSLError, "#{label} returned=1 errno=0 #{peer_text}state=error: certificate verify failed (#{reason})"
        end
        advance(OpenSSL.__tls__(:decide, @connection, nil), label)
      end

      # Up to `length` bytes of application data, reading records from the
      # IO until some arrive: nil at the end of the stream, and
      # :wait_readable when not blocking and the IO has nothing to read.
      def plaintext(length, blocking, label)
        while @plaintext.empty?
          return nil if @peer_closed || @connection.nil?
          bytes = read_io(blocking)
          return bytes if bytes == :wait_readable
          if bytes.nil?
            @peer_closed = true
            return nil
          end
          advance(OpenSSL.__tls__(:feed, @connection, bytes), label)
        end
        @plaintext.slice!(0, length)
      end

      def peer_text
        address = @io.remote_address
        return "" unless address.ip?
        "peeraddr=#{address.ipv6? ? "[#{address.ip_address}]" : address.ip_address}:#{address.ip_port} "
      rescue StandardError
        ""
      end
    end

    # A TCP server whose connections are SSLSockets.
    class SSLServer
      include SocketForwarder

      attr_accessor :start_immediately

      def initialize(server, context)
        @svr = server
        @ctx = context
        unless context.session_id_context
          context.session_id_context = ::Random.new($0.hash).bytes(16).unpack1("H*")
        end
        @start_immediately = true
      end

      def to_io
        @svr
      end

      def listen(backlog = Socket::SOMAXCONN)
        @svr.listen(backlog)
      end

      def shutdown(how = Socket::SHUT_RDWR)
        @svr.shutdown(how)
      end

      def accept
        socket, = @svr.accept
        begin
          ssl = OpenSSL::SSL::SSLSocket.new(socket, @ctx)
          ssl.sync_close = true
          ssl.accept if @start_immediately
          ssl
        rescue Exception => error
          ssl ? ssl.close : socket.close
          raise error
        end
      end

      def close
        @svr.close
      end
    end
  end
end

module OpenSSL
  module ASN1
    TAG_CLASS_BITS = { UNIVERSAL: 0x00, APPLICATION: 0x40, CONTEXT_SPECIFIC: 0x80, PRIVATE: 0xc0 }.freeze

    # A value with its tag, the way DER writes it. A constructed one holds a
    # list of values, and a primitive one its content bytes.
    class ASN1Data
      attr_accessor :value, :tag, :tag_class, :indefinite_length

      def initialize(value, tag, tag_class)
        unless TAG_CLASS_BITS.key?(tag_class)
          raise ASN1Error, "invalid tag class"
        end
        @value = value
        @tag = tag
        @tag_class = tag_class
        @indefinite_length = false
      end

      def to_der
        constructed = @value.is_a?(Array)
        content = constructed ? @value.map(&:to_der).join.b : @value.to_s.b
        ASN1.__tagged__(@tag, @tag_class, constructed, content)
      end
    end

    # A universal value read or written as one piece of content.
    class Primitive < ASN1Data
      attr_accessor :tagging

      def initialize(value, tag = nil, tagging = nil, tag_class = nil)
        tag_class ||= tag.nil? ? :UNIVERSAL : :CONTEXT_SPECIFIC
        super(value, tag || self.class::DEFAULT_TAG, tag_class)
        @tagging = tagging
      end

      def to_der
        content = __content__
        return ASN1.__tagged__(@tag, @tag_class, false, content) unless @tagging == :EXPLICIT
        inner = ASN1.__tagged__(self.class::DEFAULT_TAG, :UNIVERSAL, false, content)
        ASN1.__tagged__(@tag, @tag_class, true, inner)
      end

      def __content__
        @value.to_s.b
      end
    end

    # A universal value made of other values, such as a SEQUENCE.
    class Constructive < ASN1Data
      include Enumerable
      attr_accessor :tagging

      def initialize(value, tag = nil, tagging = nil, tag_class = nil)
        tag_class ||= tag.nil? ? :UNIVERSAL : :CONTEXT_SPECIFIC
        super(value, tag || self.class::DEFAULT_TAG, tag_class)
        @tagging = tagging
      end

      def each(&block)
        return to_enum(:each) unless block
        @value.each(&block)
        self
      end

      def to_der
        content = @value.map(&:to_der).join.b
        return ASN1.__tagged__(@tag, @tag_class, true, content) unless @tagging == :EXPLICIT
        inner = ASN1.__tagged__(self.class::DEFAULT_TAG, :UNIVERSAL, true, content)
        ASN1.__tagged__(@tag, @tag_class, true, inner)
      end
    end

    class Sequence < Constructive
      DEFAULT_TAG = SEQUENCE
    end

    class Set < Constructive
      DEFAULT_TAG = SET
    end

    class Boolean < Primitive
      DEFAULT_TAG = BOOLEAN

      def __content__
        @value ? "\xFF".b : "\x00".b
      end
    end

    class Integer < Primitive
      DEFAULT_TAG = INTEGER

      def __content__
        ASN1.__signed_bytes__(@value.to_i)
      end
    end

    class Enumerated < Primitive
      DEFAULT_TAG = ENUMERATED

      def __content__
        ASN1.__signed_bytes__(@value.to_i)
      end
    end

    class BitString < Primitive
      DEFAULT_TAG = BIT_STRING
      attr_accessor :unused_bits

      def initialize(*)
        super
        @unused_bits = 0
      end

      def __content__
        @unused_bits.chr.b + @value.to_s.b
      end
    end

    class Null < Primitive
      DEFAULT_TAG = NULL

      def initialize(value = nil, *rest)
        super
      end

      def __content__
        "".b
      end
    end

    # An object identifier, held by its short name when OpenSSL knows one.
    class ObjectId < Primitive
      DEFAULT_TAG = OBJECT

      def sn
        found = OpenSSL.__object__(:find, @value)
        found ? found[0] : nil
      end

      def ln
        found = OpenSSL.__object__(:find, @value)
        found ? found[1] : nil
      end

      def oid
        found = OpenSSL.__object__(:find, @value)
        found ? found[2] : @value
      end

      def __content__
        OpenSSL.__object__(:encode, oid)
      end
    end

    # A time written as UTCTime or GeneralizedTime.
    class UTCTime < Primitive
      DEFAULT_TAG = UTCTIME

      def __content__
        @value.utc.strftime("%y%m%d%H%M%SZ")
      end
    end

    class GeneralizedTime < Primitive
      DEFAULT_TAG = GENERALIZEDTIME

      def __content__
        @value.utc.strftime("%Y%m%d%H%M%SZ")
      end
    end

    {
      OctetString: OCTET_STRING, UTF8String: UTF8STRING, NumericString: NUMERICSTRING,
      PrintableString: PRINTABLESTRING, T61String: T61STRING, VideotexString: VIDEOTEXSTRING,
      IA5String: IA5STRING, GraphicString: GRAPHICSTRING, ISO64String: ISO64STRING,
      GeneralString: GENERALSTRING, UniversalString: UNIVERSALSTRING, BMPString: BMPSTRING,
      ObjectDescriptor: OBJECT_DESCRIPTOR
    }.each do |named, tag|
      const_set(named, Class.new(Primitive)).const_set(:DEFAULT_TAG, tag)
    end

    UNIVERSAL_CLASSES = {
      BOOLEAN => Boolean, INTEGER => Integer, BIT_STRING => BitString, OCTET_STRING => OctetString,
      NULL => Null, OBJECT => ObjectId, OBJECT_DESCRIPTOR => ObjectDescriptor, ENUMERATED => Enumerated,
      UTF8STRING => UTF8String, SEQUENCE => Sequence, SET => Set, NUMERICSTRING => NumericString,
      PRINTABLESTRING => PrintableString, T61STRING => T61String, VIDEOTEXSTRING => VideotexString,
      IA5STRING => IA5String, UTCTIME => UTCTime, GENERALIZEDTIME => GeneralizedTime,
      GRAPHICSTRING => GraphicString, ISO64STRING => ISO64String, GENERALSTRING => GeneralString,
      UNIVERSALSTRING => UniversalString, BMPSTRING => BMPString
    }.freeze

    # `ASN1::Sequence([...])` builds a `Sequence` the way `Sequence.new` does,
    # for each universal class.
    UNIVERSAL_CLASSES.each_value do |universal|
      define_singleton_method(universal.name.split("::").last) { |*arguments| universal.new(*arguments) }
    end

    # The identifier and length octets of a value, then its content.
    def self.__tagged__(tag, tag_class, constructed, content)
      raise ASN1Error, "tag number too large" if tag > 30
      leading = TAG_CLASS_BITS.fetch(tag_class) | (constructed ? 0x20 : 0) | tag
      __encoded__(leading, content)
    end

    # Two's complement, in as few bytes as hold the value and its sign.
    def self.__signed_bytes__(number)
      return __integer_bytes__(number).then { |bytes| bytes.getbyte(0) >= 0x80 ? "\x00".b + bytes : bytes } unless number.negative?
      width = 1
      width += 1 while number < -(1 << (8 * width - 1))
      __integer_bytes__((1 << (8 * width)) + number).rjust(width, "\xFF".b)
    end

    def self.__signed_value__(content)
      held = content.unpack1("H*").to_i(16)
      return held if content.empty? || content.getbyte(0) < 0x80
      held - (1 << (8 * content.bytesize))
    end

    def self.decode(der)
      der = der.to_der if der.respond_to?(:to_der)
      der = der.b
      decoded, rest = __decode_one__(der)
      raise ASN1Error, "too long" unless rest.empty?
      decoded
    end

    def self.decode_all(der)
      rest = der.b
      found = []
      until rest.empty?
        decoded, rest = __decode_one__(rest)
        found << decoded
      end
      found
    end

    def self.__decode_one__(bytes)
      held = __read_der__(bytes)
      raise ASN1Error, "header too long" if held.nil?
      leading, content, rest = held
      raise ASN1Error, "tag number too large" if (leading & 0x1f) == 0x1f
      tag = leading & 0x1f
      constructed = (leading & 0x20) != 0
      tag_class = TAG_CLASS_BITS.key(leading & 0xc0)
      [__decoded_value__(tag, tag_class, constructed, content), rest]
    end

    def self.__decoded_value__(tag, tag_class, constructed, content)
      if constructed
        children = decode_all(content)
        made = UNIVERSAL_CLASSES[tag] if tag_class == :UNIVERSAL
        return ASN1Data.new(children, tag, tag_class) unless made && made <= Constructive
        return made.new(children)
      end
      made = UNIVERSAL_CLASSES[tag] if tag_class == :UNIVERSAL
      return ASN1Data.new(content, tag, tag_class) if made.nil? || made <= Constructive
      value = case tag
              when BOOLEAN then content != "\x00".b
              when INTEGER, ENUMERATED then OpenSSL::BN.new(__signed_value__(content))
              when NULL then nil
              when OBJECT
                dotted = __oid_text__(content)
                found = OpenSSL.__object__(:find, dotted)
                found ? found[0] : dotted
              when BIT_STRING then content.byteslice(1..)
              when UTCTIME, GENERALIZEDTIME then __time_value__(tag, content)
              else content
              end
      decoded = made.new(value)
      decoded.unused_bits = content.getbyte(0) if tag == BIT_STRING
      decoded
    end
  end
end

module OpenSSL
  class ConfigError < OpenSSL::OpenSSLError; end

  # The sections of an OpenSSL configuration file, each a table of names and
  # values. Names before the first section header go in "default".
  class Config
    include Enumerable

    DEFAULT_CONFIG_FILE = "/etc/ssl/openssl.cnf"

    def self.parse(text)
      new.tap { |config| config.__read__(text) }
    end

    def self.load(path = nil)
      new(path)
    end

    def self.parse_config(io)
      parse(io.read).__sections__
    end

    def initialize(path = nil)
      @sections = {}
      __read__(File.read(path)) if path
    end

    def __read__(text)
      section = "default"
      text.each_line do |line|
        line = line.sub(/(?<!\\)#.*/, "").strip
        next if line.empty?
        if (header = line[/\A\[\s*([^\]]*?)\s*\]\z/, 1])
          section = header
          @sections[section] ||= {}
          next
        end
        name, value = line.split("=", 2)
        raise ConfigError, "missing equal sign" if value.nil?
        (@sections[section] ||= {})[name.strip] = value.strip
      end
      self
    end

    def __sections__
      @sections
    end

    def [](section)
      (@sections[section] || {}).dup
    end

    def get_value(section, name)
      @sections.fetch(section, {})[name]
    end

    def sections
      @sections.keys.sort
    end

    def each
      return to_enum(:each) unless block_given?
      @sections.each { |section, values| values.each { |name, value| yield section, name, value } }
      self
    end

    def to_s
      sections.map do |section|
        "[ #{section} ]\n" + @sections[section].map { |name, value| "#{name}=#{value}\n" }.join + "\n"
      end.join
    end

    def inspect
      "#<#{self.class.name} sections=#{sections.inspect}>"
    end
  end
end
