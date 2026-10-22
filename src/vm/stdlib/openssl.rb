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

  # A digest keyed with a secret, which says both what the message is and
  # that whoever wrote it held the key.
  class HMAC
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
      %w[aes-128-cbc aes-192-cbc aes-256-cbc]
    end

    attr_reader :name, :key_len

    def initialize name
      @name = name.to_s.upcase
      bits = @name[/\AAES-(128|192|256)-CBC\z/, 1]
      raise CipherError, "unsupported cipher algorithm (#{name})" if bits.nil?
      @key_len = KEY_BYTES[bits]
      @padding = 1
      @encrypting = true
      @iv = ("\0" * BLOCK).b
      @chain = @iv
      @pending = "".b
    end

    def iv_len = BLOCK

    def block_size = BLOCK

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
      @pending << data.b
      whole = @pending.bytesize / BLOCK * BLOCK
      # Decrypting holds the last block back, since `final` takes its padding off.
      whole -= BLOCK if !@encrypting && @padding != 0 && whole == @pending.bytesize && whole.positive?
      run whole
    end

    def final
      raise CipherError, "key not set" if @key.nil?
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
      self
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

      def canonical_encoding
        @entries.map do |entry|
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

    class PKey; end

    # The key PEM or DER text holds. RSA keys are the kind read.
    def self.read(text, _password = nil)
      text = text.read if text.respond_to?(:read)
      RSA.new(text)
    rescue PKeyError
      raise PKeyError, "Could not parse PKey"
    end

    # An RSA key pair, or the public half of one.
    class RSA < PKey
      class RSAError < PKeyError; end

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
          parts = RSA.__read__(bits)
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

      PEM_LABELS = ["RSA PRIVATE KEY", "PUBLIC KEY", "RSA PUBLIC KEY"].freeze

      # The parts of a key written as PKCS #1 RSAPrivateKey, as a
      # SubjectPublicKeyInfo, or as PKCS #1 RSAPublicKey, in DER or in PEM.
      def self.__read__(text)
        der = text.b
        if text.include?("-----BEGIN ")
          matched = text.match(/-----BEGIN (#{PEM_LABELS.join("|")})-----(.*?)-----END \1-----/m)
          return nil if matched.nil?
          der = matched[2].unpack1("m")
        end
        top = OpenSSL::ASN1.__read_der__(der)
        return nil unless top && top[0] == 0x30
        fields = OpenSSL::ASN1.__read_all__(top[1])
        return nil if fields.nil?
        if fields.size == 9 && fields.all? { |tag, _| tag == OpenSSL::ASN1::INTEGER }
          return fields.drop(1).map { |_, content| OpenSSL::ASN1.__integer_value__(content) }
        end
        if fields.size == 2 && fields.all? { |tag, _| tag == OpenSSL::ASN1::INTEGER }
          return fields.map { |_, content| OpenSSL::ASN1.__integer_value__(content) }
        end
        if fields.size == 2 && fields[0][0] == 0x30 && fields[1][0] == OpenSSL::ASN1::BIT_STRING
          inner = OpenSSL::ASN1.__read_der__(fields[1][1].byteslice(1..))
          numbers = inner && OpenSSL::ASN1.__read_all__(inner[1])
          return nil unless numbers && numbers.size == 2
          return numbers.map { |_, content| OpenSSL::ASN1.__integer_value__(content) }
        end
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
        message = encoded.unpack1("H*").to_i(16)
        # The two halves of the private operation, joined by the Chinese
        # remainder theorem.
        first = message.pow(@dmp1, @p)
        second = message.pow(@dmq1, @q)
        joined = second + ((@iqmp * (first - second)) % @p) * @q
        OpenSSL::ASN1.__integer_bytes__(joined).rjust(width, "\x00".b)
      end

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
    V_ERR_HOSTNAME_MISMATCH = 62

    VERIFY_ERROR_STRINGS = {
      0 => "ok",
      7 => "certificate signature failure",
      9 => "certificate is not yet valid or the system clock is incorrect",
      10 => "certificate has expired",
      18 => "self-signed certificate",
      19 => "self-signed certificate in certificate chain",
      20 => "unable to get local issuer certificate",
      62 => "hostname mismatch"
    }.freeze

    # One extension: the object it names, the DER of its value, and whether
    # a reader that does not know it must refuse the certificate.
    class Extension
      attr_reader :oid

      def initialize(oid, value, critical = false, shown = nil)
        @oid = oid
        @der_value = value.b
        @critical = critical
        @shown = shown
      end

      def critical?
        @critical
      end

      def value
        @shown || @der_value.unpack1("H*").scan(/../).map(&:upcase).join(":")
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
        "#{@oid} = #{"critical, " if @critical}#{value}"
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
        @public_key = OpenSSL::PKey::RSA.new(OpenSSL::ASN1.__encoded__(0x30, fields.shift[1]))
        fields.each do |tag, content|
          next unless tag == 0xa3
          listed = OpenSSL::ASN1.__read_der__(content)[1]
          @extensions = OpenSSL::ASN1.__read_all__(listed).map { |_, extension| Certificate.__extension__(extension) }
        end
        oid = OpenSSL::ASN1.__oid_text__(OpenSSL::ASN1.__read_all__(algorithm).first[1])
        @digest_name, (@signature_algorithm, @signature_oid) = SIGNATURE_ALGORITHMS.find { |_, (_, held)| held == oid }
        @signature = signature.byteslice(1..)
      end
      private :__read__

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

      def ==(other)
        other.is_a?(Certificate) && to_der == other.to_der
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

    # The certificates trusted to anchor a chain, and the result of the last
    # verification made against them.
    class Store
      attr_reader :error, :chain
      attr_accessor :time, :flags, :purpose, :trust, :verify_callback

      def initialize
        @trusted = []
        @error = 0
        @chain = nil
      end

      def add_cert(certificate)
        @trusted << certificate
        self
      end

      def __trusted__
        @trusted
      end

      def set_default_paths
        self
      end

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

      # Build the chain from the certificate up to one the store trusts,
      # finding each issuer by name and signature, and then check every
      # certificate's validity period, the root's first.
      def verify(certificate, untrusted = nil)
        candidates = @trusted + untrusted.to_a
        chain = [certificate]
        current = certificate
        loop do
          if current.__self_issued__ && current.verify(current.public_key)
            unless @trusted.include?(current)
              return failed(chain, chain.size == 1 ? V_ERR_DEPTH_ZERO_SELF_SIGNED_CERT : V_ERR_SELF_SIGNED_CERT_IN_CHAIN)
            end
            break
          end
          issuer = candidates.find do |held|
            !held.equal?(current) && held.subject.cmp(current.issuer).zero? && current.verify(held.public_key)
          end
          return failed(chain, V_ERR_UNABLE_TO_GET_ISSUER_CERT_LOCALLY) if issuer.nil?
          chain << issuer
          current = issuer
          break if @trusted.include?(issuer) && !issuer.__self_issued__
        end
        now = @time || Time.now
        chain.reverse_each do |held|
          case held.__valid_at__(now)
          when :not_yet then return failed(chain, V_ERR_CERT_NOT_YET_VALID)
          when :expired then return failed(chain, V_ERR_CERT_HAS_EXPIRED)
          end
        end
        @chain = chain
        @error = V_OK
        true
      end

      private

      def failed(chain, code)
        @chain = chain
        @error = code
        false
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

      def ciphers=(list)
        @ciphers = list
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

      # How many bytes are read from the IO at a time.
      READ_SIZE = 16_384

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
        @io.write(records)
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
          @io.write(OpenSSL.__tls__(:close, @connection)) unless @peer_closed
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
        if @connection.nil?
          @mode = mode
          @connection = if mode == :client
                          OpenSSL.__tls__(:client, @hostname)
                        else
                          key = @context.key
                          raise SSLError, "#{label} returned=1 errno=0 state=error: no shared cipher" if @context.cert.nil? || key.nil?
                          chain = [@context.cert, *@context.extra_chain_cert].map(&:to_der)
                          OpenSSL.__tls__(:server, chain, key.n.to_i, key.e.to_i, key.d.to_i)
                        end
          @state = :handshake
          opening = OpenSSL.__tls__(:start, @connection)
          @io.write(opening) unless opening.empty?
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

      def read_io(blocking)
        if blocking
          @io.readpartial(READ_SIZE)
        else
          @io.read_nonblock(READ_SIZE, exception: false)
        end
      rescue EOFError
        nil
      end

      # Send what the connection answered, keep the data it read, and carry
      # on through a chain decision or a failure.
      def advance(progress, label)
        outgoing, read, closed, state, problem = progress
        @io.write(outgoing) unless outgoing.empty?
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
        store = @context.__trusted_store__
        trusted = store.verify(@peer_cert_chain.first, @peer_cert_chain.drop(1))
        @verify_result = store.error
        if trusted && @context.verify_hostname && @hostname &&
           !OpenSSL::SSL.verify_certificate_identity(@peer_cert_chain.first, @hostname)
          trusted = false
          @verify_result = OpenSSL::X509::V_ERR_HOSTNAME_MISMATCH
        end
        if !trusted && (@context.verify_mode.to_i & OpenSSL::SSL::VERIFY_PEER) != 0
          alert = REFUSAL_ALERTS.fetch(@verify_result, 46)
          outgoing, = OpenSSL.__tls__(:decide, @connection, alert)
          @io.write(outgoing) unless outgoing.empty?
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
