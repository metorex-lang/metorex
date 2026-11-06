# Escaping text for a URL and for a page. A URL carries only a few characters
# as themselves, and a page reads five of them as markup, so both need what
# is written into them spelled out.

module CGI
  # `escape` writes a form value: every byte but those of the unreserved
  # characters is written in hex, and a space is written as a plus. What it
  # writes is in the encoding of the text it was given.
  def self.escape(text)
    held = CGI.coerced(text)
    written = CGI.escaped_bytes(held, RESERVED_OR_SPACE).tr(" ", "+")
    written.force_encoding(held.encoding)
  end

  # `unescape` reads a form value, where a plus stands for a space.
  def self.unescape(text, encoding = nil)
    CGI.unescapeURIComponent(CGI.coerced(text).tr("+", " "), encoding)
  end

  # `escapeURIComponent` writes a piece of a URL, where a space is written in
  # hex like anything else and the tilde stands for itself.
  def self.escapeURIComponent(text)
    held = CGI.coerced(text)
    CGI.escaped_bytes(held, RESERVED).force_encoding(held.encoding)
  end

  # The bytes the URI syntax does not leave alone, with and without the
  # space a form value writes as a plus.
  RESERVED = /[^a-zA-Z0-9_\-.~]/n
  RESERVED_OR_SPACE = /[^a-zA-Z0-9_\-.~ ]/n

  # The text's bytes with each one `pattern` matches written in hex. The
  # text is read as bytes, which is what carries a run that spells no
  # character through unchanged.
  def self.escaped_bytes(held, pattern)
    held.b.gsub(pattern) { |byte| "%%%02X" % byte.ord }
  end

  # The encoding a decoded string is tagged with when the call names none.
  @@accept_charset = "UTF-8"

  def self.unescapeURIComponent(text, encoding = nil)
    held = CGI.coerced(text)
    wanted = encoding.nil? ? CGI.class_variable_get(:@@accept_charset) : encoding
    target = wanted.is_a?(Encoding) ? wanted : Encoding.find(wanted.to_s)
    written = CGI.read_percent_bytes(held)
    tagged = written.dup.force_encoding(target)
    return tagged if tagged.valid_encoding?
    # Bytes that spell nothing in the encoding asked for keep the encoding
    # the text they came from was tagged with.
    written.dup.force_encoding(held.encoding)
  end

  # Read every `%NN` back into the byte it names, a byte at a time, so a run
  # that spells no character is carried through as it stands.
  def self.read_percent_bytes(held)
    source = held.bytes
    written = []
    at = 0
    while at < source.length
      if source[at] == 37 && at + 2 < source.length &&
         CGI.hex_digit?(source[at + 1]) && CGI.hex_digit?(source[at + 2])
        written.push((source[at + 1].chr + source[at + 2].chr).to_i(16))
        at = at + 3
      else
        written.push source[at]
        at = at + 1
      end
    end
    return "".dup.force_encoding(Encoding::BINARY) if written.empty?
    written.map { |byte| byte.chr }.join
  end

  def self.hex_digit?(byte)
    (48..57).include?(byte) || (65..70).include?(byte) || (97..102).include?(byte)
  end

  # The five characters a page reads as markup.
  # The five characters HTML reads as markup, written out as entities. The
  # text is read one byte at a time, so bytes that spell no character in the
  # encoding it is tagged with are carried through as they stand.
  def self.escapeHTML(text)
    held = CGI.coerced text
    written = []
    held.bytes.each do |byte|
      case byte
      when 38 then written.concat "&amp;".bytes
      when 60 then written.concat "&lt;".bytes
      when 62 then written.concat "&gt;".bytes
      when 34 then written.concat "&quot;".bytes
      when 39 then written.concat "&#39;".bytes
      else written.push byte
      end
    end
    made = written.pack "C*"
    made.force_encoding held.encoding
  end

  # The same five read back, along with a character named by its number.
  def self.unescapeHTML(text, _encoding = nil)
    held = CGI.coerced text
    return held unless held.include? "&"
    answer = ""
    at = 0
    while at < held.length
      piece = held[at]
      if piece != "&"
        answer = answer + piece
        at = at + 1
        next
      end
      finish = held.index ";", at
      named = finish.nil? ? nil : held[at + 1, finish - at - 1]
      read = CGI.named_character named
      if read.nil?
        answer = answer + piece
        at = at + 1
      else
        answer = answer + read
        at = finish + 1
      end
    end
    answer
  end

  # What one `&...;` stands for, or nil when it names nothing.
  def self.named_character(named)
    return nil if named.nil? || named.empty?
    case named
    when "amp" then "&"
    when "lt" then "<"
    when "gt" then ">"
    when "quot" then "\""
    when "apos" then "'"
    else
      return CGI.numbered_character named[1..-1] if named.start_with? "#"
      nil
    end
  end

  def self.numbered_character(digits)
    return nil if digits.nil? || digits.empty?
    if digits.start_with?("x") || digits.start_with?("X")
      rest = digits[1..-1]
      return nil unless rest =~ /\A[0-9a-fA-F]+\z/
      return rest.to_i(16).chr
    end
    return nil unless digits =~ /\A[0-9]+\z/
    digits.to_i.chr
  end

  # `escapeElement` escapes only the tags of the elements it is given, so the
  # rest of the markup is left standing.
  def self.escapeElement(text, *elements)
    named = elements.flatten.map { |held| held.to_s.downcase }
    CGI.walk_tags(CGI.coerced(text)) do |tag, name|
      named.include?(name) ? CGI.escapeHTML(tag) : tag
    end
  end

  def self.unescapeElement(text, *elements)
    named = elements.flatten.map { |held| held.to_s.downcase }
    held = CGI.coerced text
    answer = ""
    at = 0
    while at < held.length
      if held[at, 4] == "&lt;"
        finish = held.index "&gt;", at
        if finish.nil?
          answer = answer + held[at]
          at = at + 1
          next
        end
        piece = held[at, finish - at + 4]
        read = CGI.unescapeHTML piece
        name = CGI.tag_name read
        answer = answer + (named.include?(name) ? read : piece)
        at = finish + 4
      else
        answer = answer + held[at]
        at = at + 1
      end
    end
    answer
  end

  # Walk the tags in a piece of markup, handing each to the block along with
  # the name of the element it opens or closes.
  def self.walk_tags(held)
    answer = ""
    at = 0
    while at < held.length
      if held[at] == "<"
        finish = held.index ">", at
        if finish.nil?
          answer = answer + held[at..-1]
          break
        end
        tag = held[at, finish - at + 1]
        answer = answer + yield(tag, CGI.tag_name(tag))
        at = finish + 1
      else
        answer = answer + held[at]
        at = at + 1
      end
    end
    answer
  end

  # The element a tag opens or closes, named in lower case.
  def self.tag_name(tag)
    named = tag.sub("<", "").sub(">", "")
    named = named[1..-1] if named.start_with? "/"
    named.split(/[\s\/]/).first.to_s.downcase
  end

  def self.coerced(text)
    return text if text.is_a? ::String
    unless text.respond_to? :to_str
      named = case text
      when nil then "nil"
      when true then "true"
      when false then "false"
      else text.class.to_s
      end
      raise TypeError, "no implicit conversion of #{named} into String"
    end
    text.to_str
  end
end
