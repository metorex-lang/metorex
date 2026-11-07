require "uri"

escaped = URI::RFC2396_PARSER.unescape("Dij%8aat")
p escaped.bytes, escaped.encoding

tagged = escaped.dup.force_encoding("ISO-2022-JP")
p tagged.valid_encoding?
p tagged.encode(Encoding::UTF_8, undef: :replace, invalid: :replace, replace: "?")

begin
  tagged.encode(Encoding::UTF_8)
rescue => error
  p error.class, error.message
end

kanji = "\e$B$\"\e(Ba".dup.force_encoding("ISO-2022-JP")
p kanji.encode(Encoding::UTF_8)
p "\e$B$".dup.force_encoding("ISO-2022-JP").encode(Encoding::UTF_8, invalid: :replace, replace: "?")
p "a\xFFb\e$B$\"\xFF\e(B".dup.force_encoding("ISO-2022-JP").encode(Encoding::UTF_8, invalid: :replace, replace: "?")

cases = ["a\x8Ab", "\e$B$\"\x8A\e(B", "\e$B$", "\e$B$\x0A\e(B", "\e$B\x0A\e(B", "\e$B\x7F!\e(B", "\e$B$\x7F\e(B",
         "a\eb", "a\e$Ab", "\x0Ea", "\x0Fa", "\e$B\e(B", "\e$B$\"\e(Jx", "\e", "\e$", "\e$B!\x80\e(B", "\e$B\"/\e(B", "\e$B$\"\e$B$\"\e(B"]
cases.each do |bytes|
  text = bytes.dup.force_encoding("ISO-2022-JP")
  shown = begin
    text.encode("UTF-8").inspect
  rescue => error
    [error.class, error.message, error.respond_to?(:error_bytes) ? error.error_bytes&.bytes : error.error_char.bytes,
     error.respond_to?(:readagain_bytes) ? error.readagain_bytes&.bytes : nil].inspect
  end
  replaced = text.encode("UTF-8", invalid: :replace, undef: :replace, replace: "?") rescue $!.class
  puts "#{bytes.inspect} => #{shown} | #{replaced.inspect}"
end

["UTF-8", "UTF-16LE"].each do |wanted|
  ["\e$B\"/\e(B", "a\x8A"].each do |bytes|
    bytes.dup.force_encoding("ISO-2022-JP").encode(wanted)
  rescue => error
    p [wanted, error.class, error.message, error.source_encoding_name, error.destination_encoding_name,
       (error.error_char.encoding if error.respond_to?(:error_char))]
  end
end
