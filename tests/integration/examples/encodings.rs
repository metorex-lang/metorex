use super::run_example;

/// The expected output of both `encodings/converter_stepwise` variants, which
/// differ only in whether the calls are written with parentheses.
const CONVERTER_STEPWISE_OUTPUT: &str = ":finished\n\"glark\"\n[:finished, nil, nil, nil, nil]\nnil\n:undefined_conversion\nEncoding::UndefinedConversionError\n\"abc\"\n:incomplete_input\n[:incomplete_input, \"EUC-JP\", \"UTF-8\", \"\\xA4\", \"\"]\n:finished\n\"!!123\"\nEncoding::UndefinedConversionError\n";

#[test]
fn test_encodings_converter_stepwise_execution() {
    let output = run_example("encodings/converter_stepwise.rb");
    assert_eq!(output, CONVERTER_STEPWISE_OUTPUT);
}

#[test]
fn test_encodings_converter_stepwise_no_parens_execution() {
    let output = run_example("encodings/converter_stepwise_no_parens.rb");
    assert_eq!(output, CONVERTER_STEPWISE_OUTPUT);
}

/// The expected output of both `encodings/negotiation` variants, which differ
/// only in whether the calls are written with parentheses.
const NEGOTIATION_OUTPUT: &str = concat!(
    "#<Encoding:UTF-8>\n",
    "#<Encoding:UTF-8>\n",
    "nil\n",
    "nil\n",
    "#<Encoding:UTF-8>\n",
    "nil\n",
    "#<Encoding:US-ASCII>\n",
    "#<Encoding:UTF-8>\n",
    "#<Encoding:US-ASCII>\n",
    "#<Encoding:BINARY (ASCII-8BIT)>\n",
    "#<Encoding:UTF-8>\n",
    "#<Encoding:EUC-JP>\n"
);

#[test]
fn test_encodings_negotiation_execution() {
    let output = run_example("encodings/negotiation.rb");
    assert_eq!(output, NEGOTIATION_OUTPUT);
}

#[test]
fn test_encodings_negotiation_parens_execution() {
    let output = run_example("encodings/negotiation_parens.rb");
    assert_eq!(output, NEGOTIATION_OUTPUT);
}

/// The expected output of both `encodings/source_encoding_and_new` variants,
/// which differ only in whether the calls are written with parentheses.
const SOURCE_ENCODING_AND_NEW_OUTPUT: &str = concat!(
    "#<Encoding:US-ASCII>\n",
    "#<Encoding:BINARY (ASCII-8BIT)>\n",
    "#<Encoding:UTF-8>\n",
    "true\n",
    "false\n",
    "#<Encoding:BINARY (ASCII-8BIT)>\n",
    "#<Encoding:EUC-JP>\n",
    "\"abc\"\n",
    "TypeError\n",
);

#[test]
fn test_encodings_source_encoding_and_new_execution() {
    let output = run_example("encodings/source_encoding_and_new.rb");
    assert_eq!(output, SOURCE_ENCODING_AND_NEW_OUTPUT);
}

#[test]
fn test_encodings_source_encoding_and_new_no_parens_execution() {
    let output = run_example("encodings/source_encoding_and_new_no_parens.rb");
    assert_eq!(output, SOURCE_ENCODING_AND_NEW_OUTPUT);
}

/// The expected output of both `encodings/japanese_text` variants, which
/// differ only in whether the calls are written with parentheses.
const JAPANESE_TEXT_OUTPUT: &str = "[164, 162]\n[143, 171, 177]\n\"あ\"\n#<Encoding:EUC-JP>\n\"\\\"\\\\x{A4A2}\\\"\"\n\"\\\"a\\\\x{A4A2}b\\\"\"\n\"U+1F600 from UTF-8 to EUC-JP\"\n[128]\n[27, 36, 66, 57, 97]\n[27, 40, 66]\n";

#[test]
fn test_encodings_japanese_text_execution() {
    let output = run_example("encodings/japanese_text.rb");
    assert_eq!(output, JAPANESE_TEXT_OUTPUT);
}

#[test]
fn test_encodings_japanese_text_parens_execution() {
    let output = run_example("encodings/japanese_text_parens.rb");
    assert_eq!(output, JAPANESE_TEXT_OUTPUT);
}

/// The expected output of both `encodings/stream_encodings` variants, which
/// differ only in whether the calls are written with parentheses.
const STREAM_ENCODINGS_OUTPUT: &str =
    "#<Encoding:EUC-JP>\n[164, 162]\n\"あ\"\n#<Encoding:UTF-8>\n\"あ\"\n\"a.b\"\n";

#[test]
fn test_encodings_stream_encodings_execution() {
    let output = run_example("encodings/stream_encodings.rb");
    assert_eq!(output, STREAM_ENCODINGS_OUTPUT);
}

#[test]
fn test_encodings_stream_encodings_parens_execution() {
    let output = run_example("encodings/stream_encodings_parens.rb");
    assert_eq!(output, STREAM_ENCODINGS_OUTPUT);
}

/// The expected output of both `encodings/code_points` variants, which differ
/// only in whether the calls are written with parentheses.
const CODE_POINTS_OUTPUT: &str = "#<Encoding:US-ASCII>\n#<Encoding:BINARY (ASCII-8BIT)>\n[200]\n[227, 129, 130]\n[164, 162]\n[129, 64]\n[237, 160, 129, 237, 176, 128]\n[256, \"US-ASCII\", \"refused\"]\n[256, \"BINARY\", \"refused\"]\n[41376, \"EUC-JP\", \"refused\"]\n[128, \"SHIFT_JIS\", \"refused\"]\n[256, \"ISO-8859-9\", \"refused\"]\n[620, \"TIS-620\", \"refused\"]\n[55296, \"UTF-8\", \"refused\"]\nnil\n\"\"\nnil\nRangeError\n";

#[test]
fn test_encodings_code_points_execution() {
    let output = run_example("encodings/code_points.rb");
    assert_eq!(output, CODE_POINTS_OUTPUT);
}

#[test]
fn test_encodings_code_points_parens_execution() {
    let output = run_example("encodings/code_points_parens.rb");
    assert_eq!(output, CODE_POINTS_OUTPUT);
}

/// The expected output of both `encodings/byte_validity` variants, which
/// differ only in whether the calls are written with parentheses.
const BYTE_VALIDITY_OUTPUT: &str =
    "true\ntrue\ntrue\nfalse\nfalse\nfalse\nfalse\nfalse\nfalse\ntrue\ntrue\n[97, 221]\nfalse\n";

#[test]
fn test_encodings_byte_validity_execution() {
    let output = run_example("encodings/byte_validity.rb");
    assert_eq!(output, BYTE_VALIDITY_OUTPUT);
}

#[test]
fn test_encodings_byte_validity_parens_execution() {
    let output = run_example("encodings/byte_validity_parens.rb");
    assert_eq!(output, BYTE_VALIDITY_OUTPUT);
}

/// The expected output of both `encodings/shift_jis_text` variants, which
/// differ only in whether the calls are written with parentheses.
const SHIFT_JIS_TEXT_OUTPUT: &str = "#<Encoding:Shift_JIS>\n[147, 250, 150, 123, 140, 234]\n3\n6\ntrue\ntrue\n[162, 166, 163]\n3\ntrue\n\"日\"\n[130, 160]\nfalse\n";

#[test]
fn test_encodings_shift_jis_text_execution() {
    let output = run_example("encodings/shift_jis_text.rb");
    assert_eq!(output, SHIFT_JIS_TEXT_OUTPUT);
}

#[test]
fn test_encodings_shift_jis_text_parens_execution() {
    let output = run_example("encodings/shift_jis_text_parens.rb");
    assert_eq!(output, SHIFT_JIS_TEXT_OUTPUT);
}

/// The expected output of both `encodings/normalized_text` variants, which
/// differ only in whether the calls are written with parentheses.
const NORMALIZED_TEXT_OUTPUT: &str = "[7835, 803]\n[383, 803, 775]\n[7785]\n[115, 803, 775]\n[197]\n[937]\ntrue\nfalse\ntrue\ntrue\n[4352, 4449]\n[44032]\n[224]\n";

#[test]
fn test_encodings_normalized_text_execution() {
    let output = run_example("encodings/normalized_text.rb");
    assert_eq!(output, NORMALIZED_TEXT_OUTPUT);
}

#[test]
fn test_encodings_normalized_text_parens_execution() {
    let output = run_example("encodings/normalized_text_parens.rb");
    assert_eq!(output, NORMALIZED_TEXT_OUTPUT);
}

/// The expected output of both `encodings/japanese_streams` variants, which differ only in
/// whether the calls are written with parentheses.
const JAPANESE_STREAMS_OUTPUT: &str = "[97, 98, 99, 27, 36, 66, 36, 34, 27, 40, 66, 100, 101, 102]\ntrue\n[\"Shift_JIS\", :undefined]\n[\"EUC-JP\", [143, 171, 177]]\n[\"UTF-16BE\", [0, 233]]\n[\"IBM437\", [130]]\n[\"macCyrillic\", :undefined]\n[\"IBM720\", [130]]\n#<Encoding:EUC-JP>\n[164, 162]\nEncodingError\n";

#[test]
fn test_encodings_japanese_streams_execution() {
    let output = run_example("encodings/japanese_streams.rb");
    assert_eq!(output, JAPANESE_STREAMS_OUTPUT);
}

#[test]
fn test_encodings_japanese_streams_parens_execution() {
    let output = run_example("encodings/japanese_streams_parens.rb");
    assert_eq!(output, JAPANESE_STREAMS_OUTPUT);
}

/// The expected output of both `encodings/held_back_bytes` variants, which
/// differ only in whether the calls are written with parentheses.
const HELD_BACK_BYTES_OUTPUT: &str = concat!(
    ":invalid_byte_sequence\n",
    "[:invalid_byte_sequence, \"EUC-JP\", \"UTF-8\", \"\\xA1\", \"d\"]\n",
    "\"d\"\n\"\"\n\"d\"\n",
    ":invalid_byte_sequence\n",
    "[:invalid_byte_sequence, \"UTF-16LE\", \"UTF-8\", \"\\x00\\xD8\", \"a\\x00\"]\n",
    "[97, 0]\n"
);

#[test]
fn test_encodings_held_back_bytes_execution() {
    let output = run_example("encodings/held_back_bytes.rb");
    assert_eq!(output, HELD_BACK_BYTES_OUTPUT);
}

#[test]
fn test_encodings_held_back_bytes_parens_execution() {
    let output = run_example("encodings/held_back_bytes_parens.rb");
    assert_eq!(output, HELD_BACK_BYTES_OUTPUT);
}

/// The expected output of both `encodings/missing_converters` variants.
const MISSING_CONVERTERS_OUTPUT: &str = concat!(
    "#<Encoding:Emacs-Mule>\n",
    "#<Encoding:EUC-TW>\n",
    "\"code converter not found (ASCII-8BIT to Emacs-Mule)\"\n",
    "\"code converter not found (UTF-8 to EUC-TW)\"\n",
    "\"code converter not found (Emacs-Mule to ASCII-8BIT)\"\n",
    "\"code converter not found (UTF-8 to UTF-7)\"\n",
    "\"code converter not found (UTF-8 to xyz)\"\n",
    "#<Encoding:EUC-JP>\n",
    "false\n",
    "\"あ\"\n",
    "#<Encoding:UTF-8>\n",
    "#<Encoding:Emacs-Mule>\n",
    "\"code converter not found (ASCII-8BIT to Emacs-Mule)\"\n"
);

#[test]
fn test_encodings_missing_converters_execution() {
    let output = run_example("encodings/missing_converters.rb");
    assert_eq!(output, MISSING_CONVERTERS_OUTPUT);
}

#[test]
fn test_encodings_missing_converters_parens_execution() {
    let output = run_example("encodings/missing_converters_parens.rb");
    assert_eq!(output, MISSING_CONVERTERS_OUTPUT);
}

/// The expected output of both `encodings/replacing_while_encoding` variants.
const REPLACING_WHILE_ENCODING_OUTPUT: &str = concat!(
    "\"ab?c\"\n",
    "\"こ�\"\n",
    "\"ち��\"\n",
    "\"ちfoofoo\"\n",
    "\"B?\"\n",
    "\"Bfoo\"\n",
    "[164, 162, 63, 164, 162]\n",
    "[164, 162, 63, 164, 162]\n",
    "\"ab?c\"\n",
    "[Encoding::InvalidByteSequenceError, \"\\\"\\\\xFF\\\" on UTF-8\"]\n",
    "[Encoding::InvalidByteSequenceError, \"incomplete \\\"\\\\xE3\\\\x81\\\" on UTF-8\"]\n",
    "[Encoding::InvalidByteSequenceError, \"\\\"\\\\xE3\\\" followed by \\\"A\\\" on UTF-8\"]\n",
    "[Encoding::UndefinedConversionError, \"U+FFFD from UTF-8 to US-ASCII\"]\n",
    "\"あ\"\n",
    "#<Encoding:UTF-8>\n",
    "false\n",
    "\"?\"\n",
    "[Encoding::UndefinedConversionError, \"\\\"\\\\xC3\\\" from ASCII-8BIT to UTF-8\"]\n",
    "[Encoding::UndefinedConversionError, \"\\\"\\\\xC3\\\" to UTF-8 in conversion from ASCII-8BIT to UTF-8 to UTF-16LE\"]\n",
    "\"a�\"\n"
);

#[test]
fn test_encodings_replacing_while_encoding_execution() {
    let output = run_example("encodings/replacing_while_encoding.rb");
    assert_eq!(output, REPLACING_WHILE_ENCODING_OUTPUT);
}

#[test]
fn test_encodings_replacing_while_encoding_parens_execution() {
    let output = run_example("encodings/replacing_while_encoding_parens.rb");
    assert_eq!(output, REPLACING_WHILE_ENCODING_OUTPUT);
}

/// The expected output of both `encodings/encoding_fallbacks` variants.
const ENCODING_FALLBACKS_OUTPUT: &str = concat!(
    "\"Bbar\"\n",
    "\"Bdflt\"\n",
    "\"B[239, 191, 189]\"\n",
    "\"Bfffd\"\n",
    "\"BU+FFFD\"\n",
    "\"Blookup\"\n",
    "\"Bword\"\n",
    "\"Bfoo\"\n",
    "[Encoding::UndefinedConversionError, \"U+FFFD from UTF-8 to US-ASCII\"]\n",
    "[Encoding::UndefinedConversionError, \"U+FFFD from UTF-8 to US-ASCII\"]\n",
    "[TypeError, \"no implicit conversion of Object into String\"]\n",
    "[ArgumentError, \"too big fallback string\"]\n"
);

#[test]
fn test_encodings_encoding_fallbacks_execution() {
    let output = run_example("encodings/encoding_fallbacks.rb");
    assert_eq!(output, ENCODING_FALLBACKS_OUTPUT);
}

#[test]
fn test_encodings_encoding_fallbacks_parens_execution() {
    let output = run_example("encodings/encoding_fallbacks_parens.rb");
    assert_eq!(output, ENCODING_FALLBACKS_OUTPUT);
}

/// The expected output of both `encodings/xml_escaping` variants.
const XML_ESCAPING_OUTPUT: &str = concat!(
    "\"&lt;a &amp; b&gt;\"\n",
    "\"say \\\"hi\\\"\"\n",
    "\"&lt;ü&gt;\"\n",
    "\"\\\"&lt;&#xFC;&gt;&quot;\\\"\"\n",
    "\"&#x1F600;&amp;\"\n",
    "[34, 0, 38, 0, 108, 0, 116, 0, 59, 0, 34, 0]\n",
    "\"&#xFC;\"\n",
    "\"&#xFC;\"\n",
    "\"unexpected value for xml option: other\"\n",
    "\"unexpected value for xml option\"\n"
);

#[test]
fn test_encodings_xml_escaping_execution() {
    let output = run_example("encodings/xml_escaping.rb");
    assert_eq!(output, XML_ESCAPING_OUTPUT);
}

#[test]
fn test_encodings_xml_escaping_parens_execution() {
    let output = run_example("encodings/xml_escaping_parens.rb");
    assert_eq!(output, XML_ESCAPING_OUTPUT);
}

/// The expected output of both `encodings/newline_conversions` variants.
const NEWLINE_CONVERSIONS_OUTPUT: &str = concat!(
    "\"a\\nb\\nc\\nd\"\n",
    "\"a\\r\\r\\nb\\rc\\r\\nd\"\n",
    "\"a\\r\\rb\\rc\\rd\"\n",
    "\"a\\r\\nb\\rc\\nd\"\n",
    "\"a\\nb\\nc\\nd\"\n",
    "\"a\\r\\r\\nb\\rc\\r\\nd\"\n",
    "\"a\\nb\\nc\\nd\"\n",
    "[Encoding::ConverterNotFoundError, \"code converter not found (universal_newline,crlf_newline)\"]\n",
    "[Encoding::ConverterNotFoundError, \"code converter not found (crlf_newline,cr_newline)\"]\n",
    "[ArgumentError, \"unexpected value for newline option: other\"]\n",
    "[ArgumentError, \"unexpected value for newline option\"]\n"
);

#[test]
fn test_encodings_newline_conversions_execution() {
    let output = run_example("encodings/newline_conversions.rb");
    assert_eq!(output, NEWLINE_CONVERSIONS_OUTPUT);
}

#[test]
fn test_encodings_newline_conversions_parens_execution() {
    let output = run_example("encodings/newline_conversions_parens.rb");
    assert_eq!(output, NEWLINE_CONVERSIONS_OUTPUT);
}

/// The expected output of both `encodings/primitive_conversions` variants.
const PRIMITIVE_CONVERSIONS_OUTPUT: &str = concat!(
    ":undefined_conversion\n",
    "\"abcd\"\n",
    "[225]\n",
    "#<Encoding:ISO-8859-1>\n",
    ":destination_buffer_full\n",
    ":destination_buffer_full\n",
    ":finished\n",
    "\"aabbb\"\n",
    ":finished\n",
    "\"  \"\n",
    "\"output_byteoffset too big\"\n",
    ":invalid_byte_sequence\n",
    "\"bcd\"\n",
    "[:invalid_byte_sequence, \"UTF-8\", \"ISO-8859-1\", \"\\xF1\", \"a\"]\n",
    ":finished\n",
    "\"abcd\"\n",
    ":invalid_byte_sequence\n",
    "\"\\x80\"\n",
    ":source_buffer_empty\n",
    "\"\"\n",
    ":finished\n",
    "\"あ\"\n",
    ":destination_buffer_full\n",
    ":finished\n",
    "[27, 36, 66, 57, 97, 27, 40, 66]\n",
    "#<Encoding:UTF8-MAC>\n",
    "2\n",
    "[[164, 162], [97]]\n",
    "[225]\n"
);

#[test]
fn test_encodings_primitive_conversions_execution() {
    let output = run_example("encodings/primitive_conversions.rb");
    assert_eq!(output, PRIMITIVE_CONVERSIONS_OUTPUT);
}

#[test]
fn test_encodings_primitive_conversions_parens_execution() {
    let output = run_example("encodings/primitive_conversions_parens.rb");
    assert_eq!(output, PRIMITIVE_CONVERSIONS_OUTPUT);
}

/// The expected output of both `encodings/big5_source` variants.
const BIG5_SOURCE_OUTPUT: &str = "[167, 65, 166, 110]\n#<Encoding:Big5>\n#<Encoding:Big5>\n";

#[test]
fn test_encodings_big5_source_execution() {
    let output = run_example("encodings/big5_source.rb");
    assert_eq!(output, BIG5_SOURCE_OUTPUT);
}

#[test]
fn test_encodings_big5_source_no_parens_execution() {
    let output = run_example("encodings/big5_source_no_parens.rb");
    assert_eq!(output, BIG5_SOURCE_OUTPUT);
}

/// The expected output of both `encodings/regexp_encodings` variants.
const REGEXP_ENCODINGS_OUTPUT: &str = "3\n[195, 169]\n[195, 169, 98]\n[[195, 169]]\n1\n\"a\\x{82A0}\\xB1\"\n[177]\n[#<Encoding:EUC-JP>, #<Encoding:EUC-JP>, #<Encoding:Windows-31J>, #<Encoding:UTF-8>]\ntrue\n#<Encoding:BINARY (ASCII-8BIT)>\n\"incompatible encoding regexp match (US-ASCII regexp with UTF-16LE string)\"\n\"incompatible encoding regexp match (US-ASCII regexp with UTF-8 string)\"\n\"invalid byte sequence in UTF-8\"\n[\"historical binary regexp match /.../n against UTF-8 string\\n\"]\n[\"AB é \\xFF\\n\", false]\n[195, 169, 32, 255]\n[49, 32, 255]\n";

#[test]
fn test_encodings_regexp_encodings_execution() {
    let output = run_example("encodings/regexp_encodings.rb");
    assert_eq!(output, REGEXP_ENCODINGS_OUTPUT);
}

#[test]
fn test_encodings_regexp_encodings_no_parens_execution() {
    let output = run_example("encodings/regexp_encodings_no_parens.rb");
    assert_eq!(output, REGEXP_ENCODINGS_OUTPUT);
}

/// The expected output of both `encodings/symbol_encodings` variants.
const SYMBOL_ENCODINGS_OUTPUT: &str = concat!(
    "#<Encoding:US-ASCII>\n",
    "#<Encoding:UTF-8>\n",
    "#<Encoding:UTF-16LE>\n",
    "#<Encoding:UTF-16LE>\n",
    "22\n",
);

#[test]
fn test_encodings_symbol_encodings_execution() {
    let output = run_example("encodings/symbol_encodings.rb");
    assert_eq!(output, SYMBOL_ENCODINGS_OUTPUT);
}

#[test]
fn test_encodings_symbol_encodings_parens_execution() {
    let output = run_example("encodings/symbol_encodings_parens.rb");
    assert_eq!(output, SYMBOL_ENCODINGS_OUTPUT);
}

/// The expected output of both `encodings/converting_through_jis` variants.
const CONVERTING_THROUGH_JIS_OUTPUT: &str = concat!(
    "[[\"UTF-8\", \"EUC-JP\"], [\"EUC-JP\", \"stateless-ISO-2022-JP\"], [\"stateless-ISO-2022-JP\", \"ISO-2022-JP\"]]\n",
    "[[\"Shift_JIS\", \"EUC-JP\"]]\n",
    "\"\\e\\x24\\x42\\x24\\x22\"\n",
    "\"\\e\\x28\\x42\\x61\"\n",
    "\"\\e\\x24\\x42\\x24\\x24\"\n",
    "\"\\e\\x28\\x42\"\n",
    "\"\"\n",
    "\"\"\n",
    "\"あz\"\n",
    "\"\"\n",
    "\"\"\n",
    "[Encoding::InvalidByteSequenceError, \"incomplete \\\"$\\\" on ISO-2022-JP\"]\n",
    "[Encoding::InvalidByteSequenceError, \"\\\"\\\\xA4\\\" on ISO-2022-JP\"]\n",
    "[Encoding::UndefinedConversionError, \"\\\"\\\\x8F\\\\xAB\\\\xB1\\\" to stateless-ISO-2022-JP in conversion from UTF-8 to EUC-JP to stateless-ISO-2022-JP to ISO-2022-JP\"]\n",
    "[Encoding::UndefinedConversionError, \"U+20AC to EUC-JP in conversion from UTF-8 to EUC-JP to stateless-ISO-2022-JP to ISO-2022-JP\"]\n",
    "[Encoding::UndefinedConversionError, \"U+3042 to ISO-8859-1 in conversion from Shift_JIS to UTF-8 to ISO-8859-1\"]\n",
    "\"\\e\\x24\\x42\\x24\\x22\\e\\x28\\x42\\x3F\"\n",
    "\"\"\n",
    "[]\n",
    "[48, 66]\n",
    "[\"universal_newline\", [#<Encoding:UTF-8>, #<Encoding:UTF-16LE>]]\n",
    "[97, 0]\n",
    "[10, 0, 98, 0]\n",
    "[10, 0]\n",
    "[164, 162]\n",
    "\"あ\"\n",
    "false\n",
);

#[test]
fn test_encodings_converting_through_jis_execution() {
    let output = run_example("encodings/converting_through_jis.rb");
    assert_eq!(output, CONVERTING_THROUGH_JIS_OUTPUT);
}

#[test]
fn test_encodings_converting_through_jis_no_parens_execution() {
    let output = run_example("encodings/converting_through_jis_no_parens.rb");
    assert_eq!(output, CONVERTING_THROUGH_JIS_OUTPUT);
}

/// The expected output of both `encodings/conversion_error_details` variants.
const CONVERSION_ERROR_DETAILS_OUTPUT: &str = concat!(
    "sjis ff: Encoding::InvalidByteSequenceError: \"\\xFF\" on Shift_JIS [source_encoding_name=\"Shift_JIS\", destination_encoding_name=\"UTF-8\", error_bytes=\"\\xFF\", readagain_bytes=nil, incomplete_input?=false]\n",
    "sjis followed: Encoding::InvalidByteSequenceError: \"\\x82\" followed by \" \" on Shift_JIS [source_encoding_name=\"Shift_JIS\", destination_encoding_name=\"UTF-8\", error_bytes=\"\\x82\", readagain_bytes=\" \", incomplete_input?=false]\n",
    "sjis 80: Encoding::InvalidByteSequenceError: \"\\x80\" on Shift_JIS [source_encoding_name=\"Shift_JIS\", destination_encoding_name=\"UTF-8\", error_bytes=\"\\x80\", readagain_bytes=nil, incomplete_input?=false]\n",
    "euc ff: Encoding::InvalidByteSequenceError: \"\\xFF\" on EUC-JP [source_encoding_name=\"EUC-JP\", destination_encoding_name=\"UTF-8\", error_bytes=\"\\xFF\", readagain_bytes=nil, incomplete_input?=false]\n",
    "euc followed: Encoding::InvalidByteSequenceError: \"\\xA4\" followed by \"A\" on EUC-JP [source_encoding_name=\"EUC-JP\", destination_encoding_name=\"UTF-8\", error_bytes=\"\\xA4\", readagain_bytes=\"A\", incomplete_input?=false]\n",
    "euc ss3 short: Encoding::InvalidByteSequenceError: incomplete \"\\x8F\\xA1\" on EUC-JP [source_encoding_name=\"EUC-JP\", destination_encoding_name=\"UTF-8\", error_bytes=\"\\x8F\\xA1\", readagain_bytes=nil, incomplete_input?=true]\n",
    "euc ss2 bad: Encoding::InvalidByteSequenceError: \"\\x8E\" followed by \"A\" on EUC-JP [source_encoding_name=\"EUC-JP\", destination_encoding_name=\"UTF-8\", error_bytes=\"\\x8E\", readagain_bytes=\"A\", incomplete_input?=false]\n",
    "utf16 to latin1: Encoding::UndefinedConversionError: U+3042 to ISO-8859-1 in conversion from UTF-16LE to UTF-8 to ISO-8859-1 [source_encoding_name=\"UTF-8\", destination_encoding_name=\"ISO-8859-1\", error_char=\"あ\"]\n",
    "utf8 to sjis: Encoding::UndefinedConversionError: U+1F600 from UTF-8 to Shift_JIS [source_encoding_name=\"UTF-8\", destination_encoding_name=\"Shift_JIS\", error_char=\"😀\"]\n",
    "utf8 followed: Encoding::InvalidByteSequenceError: \"\\xE3\" followed by \"A\" on UTF-8 [source_encoding_name=\"UTF-8\", destination_encoding_name=\"UTF-16LE\", error_bytes=\"\\xE3\", readagain_bytes=\"A\", incomplete_input?=false]\n",
    "binary to latin1: Encoding::UndefinedConversionError: \"\\xFF\" to UTF-8 in conversion from ASCII-8BIT to UTF-8 to ISO-8859-1 [source_encoding_name=\"ASCII-8BIT\", destination_encoding_name=\"UTF-8\", error_char=\"\\xFF\"]\n",
    "sjis to eucjp: no error\n",
    "utf8 to iso2022: Encoding::UndefinedConversionError: U+1F600 to EUC-JP in conversion from UTF-8 to EUC-JP to stateless-ISO-2022-JP to ISO-2022-JP [source_encoding_name=\"UTF-8\", destination_encoding_name=\"EUC-JP\", error_char=\"😀\"]\n",
    "sjis to latin1: Encoding::UndefinedConversionError: U+3042 to ISO-8859-1 in conversion from Shift_JIS to UTF-8 to ISO-8859-1 [source_encoding_name=\"UTF-8\", destination_encoding_name=\"ISO-8859-1\", error_char=\"あ\"]\n",
    "invalid utf8: Encoding::InvalidByteSequenceError: \"\\xFF\" on UTF-8 [source_encoding_name=\"UTF-8\", destination_encoding_name=\"UTF-16LE\", error_bytes=\"\\xFF\", readagain_bytes=nil, incomplete_input?=false]\n",
    "incomplete utf8: Encoding::InvalidByteSequenceError: incomplete \"\\xE3\\x81\" on UTF-8 [source_encoding_name=\"UTF-8\", destination_encoding_name=\"UTF-16LE\", error_bytes=\"\\xE3\\x81\", readagain_bytes=nil, incomplete_input?=true]\n",
    "incomplete sjis: Encoding::InvalidByteSequenceError: incomplete \"\\x82\" on Shift_JIS [source_encoding_name=\"Shift_JIS\", destination_encoding_name=\"UTF-8\", error_bytes=\"\\x82\", readagain_bytes=nil, incomplete_input?=true]\n",
    "incomplete eucjp: Encoding::InvalidByteSequenceError: incomplete \"\\x8E\" on EUC-JP [source_encoding_name=\"EUC-JP\", destination_encoding_name=\"Shift_JIS\", error_bytes=\"\\x8E\", readagain_bytes=nil, incomplete_input?=true]\n",
    "binary to utf8: Encoding::UndefinedConversionError: \"\\xFF\" from ASCII-8BIT to UTF-8 [source_encoding_name=\"ASCII-8BIT\", destination_encoding_name=\"UTF-8\", error_char=\"\\xFF\"]\n",
    "latin1 to ascii: Encoding::UndefinedConversionError: U+00FF to US-ASCII in conversion from ISO-8859-1 to UTF-8 to US-ASCII [source_encoding_name=\"UTF-8\", destination_encoding_name=\"US-ASCII\", error_char=\"ÿ\"]\n",
);

#[test]
fn test_encodings_conversion_error_details_execution() {
    let output = run_example("encodings/conversion_error_details.rb");
    assert_eq!(output, CONVERSION_ERROR_DETAILS_OUTPUT);
}

#[test]
fn test_encodings_conversion_error_details_no_parens_execution() {
    let output = run_example("encodings/conversion_error_details_no_parens.rb");
    assert_eq!(output, CONVERSION_ERROR_DETAILS_OUTPUT);
}

/// The expected output of both `encodings/encoding_names` variants.
const ENCODING_NAMES_OUTPUT: &str = concat!(
    "US-ASCII: [\"US-ASCII\", \"ASCII\", \"ANSI_X3.4-1968\", \"646\"]\n",
    "UTF-8: [\"UTF-8\", \"CP65001\"]\n",
    "Shift_JIS: [\"Shift_JIS\"]\n",
    "ISO-2022-JP: [\"ISO-2022-JP\", \"ISO2022-JP\"]\n",
    "UTF-7: [\"UTF-7\", \"CP65000\"]\n",
    "GB2312: [\"GB2312\", \"EUC-CN\", \"eucCN\"]\n",
    "IBM720: [\"IBM720\", \"CP720\"]\n",
    "Big5-HKSCS: [\"Big5-HKSCS\", \"Big5-HKSCS:2008\"]\n",
    "646 -> US-ASCII\n",
    "eucJP -> EUC-JP\n",
    "CP932 -> Windows-31J\n",
    "ebcdic-cp-us -> IBM037\n",
    "Big5-HKSCS:2008 -> Big5-HKSCS\n",
    "EUC-CN -> GB2312\n",
    "cp720 -> IBM720\n",
    "[\"ASCII-8BIT\", \"UTF-8\", \"US-ASCII\", \"UTF-16BE\", \"UTF-16LE\", \"UTF-32BE\", \"UTF-32LE\", \"UTF-16\", \"UTF-32\", \"UTF8-MAC\", \"EUC-JP\", \"Windows-31J\"]\n",
    "[\"SJIS-SoftBank\", \"locale\", \"external\", \"filesystem\", \"internal\"]\n",
    "103\n",
    "\"US-ASCII\"\n",
    "#<Encoding:IBM720>\n",
    "#<Encoding:EUC-JP>\n",
);

#[test]
fn test_encodings_encoding_names_execution() {
    let output = run_example("encodings/encoding_names.rb");
    assert_eq!(output, ENCODING_NAMES_OUTPUT);
}

#[test]
fn test_encodings_encoding_names_no_parens_execution() {
    let output = run_example("encodings/encoding_names_no_parens.rb");
    assert_eq!(output, ENCODING_NAMES_OUTPUT);
}

/// The expected output of both `encodings/symbol_and_regexp_inspect` variants.
const SYMBOL_AND_REGEXP_INSPECT_OUTPUT: &str = concat!(
    "binary high: :\"\\xE3\\x81\\x82\" ASCII-8BIT | /\\xE3\\x81\\x82/ ASCII-8BIT | \"\\xE3\\x81\\x82\"\n",
    "binary ascii: :abc US-ASCII | /abc/ US-ASCII | \"abc\"\n",
    "sjis: :\"\\x{82A0}\" Shift_JIS | /\\x{82A0}/ Shift_JIS | \"\\x{82A0}\"\n",
    "eucjp: :\"\\x{A4A2}\" EUC-JP | /\\x{A4A2}/ EUC-JP | \"\\x{A4A2}\"\n",
    "latin1: :\"caf\\xE9\" ISO-8859-1 | /caf\\xE9/ ISO-8859-1 | \"caf\\xE9\"\n",
    "utf16le: :\"a\" UTF-16LE | /a\u{0000}/ UTF-16LE | \"a\"\n",
    "invalid utf8: #<EncodingError: invalid symbol in encoding UTF-8 :\"\\xFF\"> - | #<RegexpError: invalid multibyte character: /\\xFF/> - | -\n",
    "utf8 multi: :あ UTF-8 | /あ/ UTF-8 | \"あ\"\n",
    "binary space: :\"a b\" US-ASCII | /a b/ US-ASCII | \"a b\"\n",
    "sjis ascii: :abc US-ASCII | /abc/ US-ASCII | \"abc\"\n",
    "\"caf\\xE9\"\n",
    "1\n",
    "/ab\\/c/\n",
);

#[test]
fn test_encodings_symbol_and_regexp_inspect_execution() {
    let output = run_example("encodings/symbol_and_regexp_inspect.rb");
    assert_eq!(output, SYMBOL_AND_REGEXP_INSPECT_OUTPUT);
}

#[test]
fn test_encodings_symbol_and_regexp_inspect_no_parens_execution() {
    let output = run_example("encodings/symbol_and_regexp_inspect_no_parens.rb");
    assert_eq!(output, SYMBOL_AND_REGEXP_INSPECT_OUTPUT);
}

/// Run an example under the C locale, with `argument` as its one argument.
fn run_example_in_the_c_locale(path: &str, argument: &str) -> String {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_metorex"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg(format!("{}/{}", crate::common::EXAMPLES_DIR, path))
        .arg(argument)
        .env("LC_ALL", "C")
        .output()
        .expect("failed to execute example");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("stdout was not utf8")
}

/// The expected output of both `encodings/c_locale` variants.
const C_LOCALE_OUTPUT: &str = concat!(
    "#<Encoding:US-ASCII>\n",
    "#<Encoding:US-ASCII>\n",
    "#<Encoding:US-ASCII>\n",
    "[#<Encoding:US-ASCII>]\n",
    "[#<Encoding:BINARY (ASCII-8BIT)>, true]\n",
    "\"\\\"\\\\xC2\\\\xA9\\\"\"\n",
    "\":\\\"\\\\u00EA\\\"\"\n",
    "#<Encoding:US-ASCII>\n",
);

#[test]
fn test_encodings_c_locale_execution() {
    let output = run_example_in_the_c_locale("encodings/c_locale.rb", "arg");
    assert_eq!(output, C_LOCALE_OUTPUT);
}

#[test]
fn test_encodings_c_locale_no_parens_execution() {
    let output = run_example_in_the_c_locale("encodings/c_locale_no_parens.rb", "arg");
    assert_eq!(output, C_LOCALE_OUTPUT);
}

#[test]
fn test_utf16_strings_are_built_and_sliced_a_character_at_a_time() {
    let output = run_example("encodings/utf16_string_building.rb");
    assert_eq!(
        output,
        "[#<Encoding:UTF-16LE>, \" carté\"]\n[#<Encoding:UTF-16LE>, \" carté\"]\n\"-carté\"\n[\" a\", \"+carté\", \" b\"]\n[#<Encoding:UTF-16LE>, \" a\\n+carté\\n b\"]\n\"@@ -1 +1 @@\\n a\\n+carté\\n b\"\n[#<Encoding:UTF-16LE>, \"  carté\"]\n[\"c\", \"ar\", \"é\"]\n"
    );
}

#[test]
fn test_interpolating_utf16_text_keeps_utf16_even_for_ascii_text() {
    let output = run_example("encodings/utf16_interpolation.rb");
    assert_eq!(
        output,
        "#<Encoding:UTF-16LE>\n#<Encoding:UTF-16LE>\n#<Encoding:UTF-16LE>\n#<Encoding:UTF-16LE>\n"
    );
}

#[test]
fn test_utf16_lines_join_and_append_in_utf16() {
    let output = run_example("encodings/utf16_join.rb");
    assert_eq!(
        output,
        "[#<Encoding:UTF-16LE>, \"! carté\"]\n[#<Encoding:UTF-16LE>, \"! carté\\ncarté\"]\n\"--- 1 ----\\n! carté\"\n"
    );
}

#[test]
fn an_encoding_matches_a_pattern_of_encoding_constants() {
    let output = run_example("encodings/encoding_alternative_pattern.rb");
    assert_eq!(output, "true\ntrue\nfalse\ntrue\n");
}

#[test]
fn single_byte_encodings_read_and_write_through_their_own_tables() {
    let output = run_example("encodings/single_byte_encodings.rb");
    assert_eq!(
        output,
        "[\"ISO-8859-2\", \"ć\", [230]]\n[\"ISO-8859-3\", \"ĉ\", [230]]\n[\"ISO-8859-5\", \"ц\", [230]]\n[\"ISO-8859-7\", \"ζ\", [230]]\n[\"KOI8-R\", \"Ф\", [230]]\n[\"Windows-1251\", \"ж\", [230]]\n[\"IBM866\", \"ц\", [230]]\n[\"macRoman\", \"Ê\", [230]]\n[230]\n\"\\\"\\\\xA5\\\" from ISO-8859-3 to UTF-8\"\n[165]\n\"\\\"\\\\xA5\\\" to UTF-8 in conversion from ISO-8859-3 to UTF-8 to ISO-8859-1\"\n\"a�b\"\n\"U+0109 from UTF-8 to ISO-8859-5\"\n"
    );
}

#[test]
fn encode_naming_the_same_source_relabels_the_bytes() {
    let output = run_example("encodings/encode_naming_the_source.rb");
    assert_eq!(
        output,
        "\"rené\"\n#<Encoding:UTF-8>\ntrue\n8\n\"café\"\n#<Encoding:UTF-8>\n#<Encoding:UTF-8>\n"
    );
}

#[test]
fn a_relabeled_string_matches_its_literal_in_searches_and_hashes() {
    let output = run_example("encodings/relabeled_strings_compare.rb");
    assert_eq!(
        output,
        "#<Encoding:UTF-8>\n#<Encoding:UTF-8>\ntrue\ntrue\n0\ntrue\n[]\n1\n1\n\"rené\"\n"
    );
}

#[test]
fn a_binary_inspection_joins_into_a_message_as_its_bytes() {
    let output = run_example("encodings/format_with_binary_argument.rb");
    assert_eq!(
        output,
        "nil\n\"  \\xFF\"\n#<Encoding:BINARY (ASCII-8BIT)>\n[32, 32, 255]\n#<Encoding:BINARY (ASCII-8BIT)>\n[255, 120]\n#<Encoding:BINARY (ASCII-8BIT)>\n[97, 255]\nEncoding::UndefinedConversionError\n\"\\\"\\\\xFF\\\" from ASCII-8BIT to UTF-8\"\n[255]\nfalse\nEncoding::CompatibilityError\n\"invalid byte sequence in UTF-8\"\n[32, 32, 255]\n#<Encoding:UTF-8>\n[32, 32, 255]\n#<Encoding:UTF-8>\nfalse\n"
    );
}

#[test]
fn strip_trims_broken_utf8_only_from_a_whole_front() {
    let output = run_example("encodings/strip_broken_utf8.rb");
    assert_eq!(
        output,
        "[[:lstrip, ArgumentError, \"invalid byte sequence in UTF-8\"], [:rstrip, Encoding::CompatibilityError, \"invalid byte sequence in UTF-8\"], [:strip, ArgumentError, \"invalid byte sequence in UTF-8\"]]\n[[:lstrip, [97, 32, 255, 32], \"UTF-8\"], [:rstrip, Encoding::CompatibilityError, \"invalid byte sequence in UTF-8\"], [:strip, Encoding::CompatibilityError, \"invalid byte sequence in UTF-8\"]]\n[[:lstrip, ArgumentError, \"invalid byte sequence in UTF-8\"], [:rstrip, Encoding::CompatibilityError, \"invalid byte sequence in UTF-8\"], [:strip, ArgumentError, \"invalid byte sequence in UTF-8\"]]\n[[:lstrip, ArgumentError, \"invalid byte sequence in UTF-8\"], [:rstrip, Encoding::CompatibilityError, \"invalid byte sequence in UTF-8\"], [:strip, ArgumentError, \"invalid byte sequence in UTF-8\"]]\n[[:lstrip, [97, 255, 32, 32], \"UTF-8\"], [:rstrip, Encoding::CompatibilityError, \"invalid byte sequence in UTF-8\"], [:strip, Encoding::CompatibilityError, \"invalid byte sequence in UTF-8\"]]\n[[:lstrip, [98, 255], \"UTF-8\"], [:rstrip, Encoding::CompatibilityError, \"invalid byte sequence in UTF-8\"], [:strip, Encoding::CompatibilityError, \"invalid byte sequence in UTF-8\"]]\n\"b\\xFF\"\n[98, 255]\nFrozenError\n"
    );
}

#[test]
fn a_meta_character_in_a_binary_source_is_binary() {
    let output = run_example("encodings/meta_character_in_binary_source.rb");
    assert_eq!(
        output,
        "#<Encoding:BINARY (ASCII-8BIT)>\ntrue\n[154]\n#<Encoding:BINARY (ASCII-8BIT)>\n"
    );
}

#[test]
fn a_literal_escaping_bytes_past_ascii_in_an_ascii_source_is_binary() {
    let output = run_example("encodings/escaped_bytes_in_ascii_source.rb");
    assert_eq!(
        output,
        "#<Encoding:BINARY (ASCII-8BIT)>\n#<Encoding:US-ASCII>\n#<Encoding:BINARY (ASCII-8BIT)>\n#<Encoding:US-ASCII>\n"
    );
}

#[test]
fn a_frozen_literal_escaping_bytes_past_ascii_in_an_ascii_source_is_binary() {
    let output = run_example("encodings/escaped_bytes_in_frozen_ascii_source.rb");
    assert_eq!(
        output,
        "#<Encoding:BINARY (ASCII-8BIT)>\n#<Encoding:US-ASCII>\n"
    );
}

#[test]
fn a_method_in_an_ascii_script_writes_its_literals_in_the_script_encoding() {
    let output = run_example("encodings/method_in_ascii_script.rb");
    assert_eq!(
        output,
        "#<Encoding:BINARY (ASCII-8BIT)>\n#<Encoding:US-ASCII>\n#<Encoding:BINARY (ASCII-8BIT)>\n#<Encoding:US-ASCII>\n"
    );
}

#[test]
fn encoding_from_us_ascii_refuses_or_replaces_bytes_past_ascii() {
    let output = run_example("encodings/encode_from_ascii_with_high_bytes.rb");
    assert_eq!(
        output,
        "\"a\u{fffd}b\u{fffd}c\"\n\"a?b?c\"\n\"\\\"\\\\x80\\\" on US-ASCII\"\n\"\\x80\"\n\"US-ASCII\"\n\"UTF-8\"\n#<Encoding:UTF-8>\n"
    );
}

#[test]
fn a_match_keeps_the_encoding_of_the_string_it_was_found_in() {
    let output = run_example("encodings/matches_keep_the_subject_encoding.rb");
    assert_eq!(
        output,
        concat!(
            "[#<Encoding:ISO-8859-1>, [233], [\"e9\"]]\n",
            "\"%E9\"\n",
            "[[#<Encoding:ISO-8859-1>, [233]]]\n",
            "[[233]]\n",
            "\"UTF-8b\"\n",
        )
    );
}

#[test]
fn nfc_keeps_hangul_syllables_and_composes_conjoining_jamo() {
    let output = run_example("encodings/hangul_composition.rb");
    assert_eq!(
        output,
        "[\"c138\", \"acc4\"]\n[\"1109\", \"1166\", \"1100\", \"1168\"]\n[\"ac01\"]\n"
    );
}

#[test]
fn scan_and_gsub_over_shift_jis_text_match_whole_characters() {
    let output = run_example("encodings/shift_jis_regex_matches_whole_characters.rb");
    assert_eq!(
        output,
        "[131, 65]\n[[131, 65]]\n\"83%41\"\n#<Encoding:Shift_JIS>\n"
    );
}

#[test]
fn gbk_text_converts_and_matches_by_whole_characters() {
    let output = run_example("encodings/gbk_regex_matches_whole_characters.rb");
    assert_eq!(
        output,
        concat!(
            "[212, 83]\ntrue\n1\n\"d4%53\"\n\"\\\"\\\\x{D453}\\\"\"\n",
            "[[212, 83]]\n\"許\"\nfalse\n",
        )
    );
}

#[test]
fn an_empty_binary_string_keeps_its_encoding_when_ascii_is_appended() {
    let output = run_example("encodings/appending_to_an_empty_binary_string.rb");
    assert_eq!(
        output,
        concat!(
            "#<Encoding:BINARY (ASCII-8BIT)>\n#<Encoding:BINARY (ASCII-8BIT)>\n",
            "#<Encoding:UTF-8>\n4\n\"xé\"\nfalse\n3\n",
        )
    );
}

#[test]
fn bytes_substituted_one_at_a_time_join_into_characters_and_broken_text_refuses_case_mapping() {
    let output = run_example("encodings/gsub_pieces_that_join_into_utf8.rb");
    assert_eq!(
        output,
        concat!(
            "\"café\"\n4\ntrue\n1\ntrue\n",
            "\"input string invalid\"\n\"input string invalid\"\n\"input string invalid\"\n",
            "\"\\xBF\"\n",
        )
    );
}

#[test]
fn split_keeps_each_piece_in_its_encoding() {
    let output = run_example("encodings/split_keeps_each_piece_in_its_encoding.rb");
    assert_eq!(
        output,
        concat!(
            "[[97, 228], [98, 228]]\n",
            "[[97, 228], [98, 228]]\n",
            "[#<Encoding:ISO-8859-1>, #<Encoding:ISO-8859-1>]\n",
            "5\n",
            "[[97, 228, 32, 98, 228]]\n",
            "[[255], [254]]\n",
            "[#<Encoding:BINARY (ASCII-8BIT)>, #<Encoding:BINARY (ASCII-8BIT)>]\n",
            "[\"héllo\", \"wörld\"]\n",
            "[#<Encoding:UTF-8>, #<Encoding:UTF-8>]\n",
            "[[99, 97, 102, 233], [99, 114, 232, 109, 101]]\n",
            "[true, true]\n",
        )
    );
}

#[test]
fn single_byte_text_keeps_its_bytes_through_string_methods() {
    let output = run_example("encodings/single_byte_text_keeps_its_bytes.rb");
    assert_eq!(
        output,
        concat!(
            "[:gsub_string, [\"ISO-8859-1\", [228, 228]]]\n",
            "[:gsub_block, [\"ISO-8859-1\", [228, 228]]]\n",
            "[:sub, [\"ISO-8859-1\", [228, 228]]]\n",
            "[:index_range, [\"ISO-8859-1\", [228]]]\n",
            "[:slice_regexp, [\"ISO-8859-1\", [228]]]\n",
            "[:scan, [[], [228], [], [228], []]]\n",
            "[:chars, [[228], [], [], [], [], [], [], [], [], [], [], []]]\n",
            "[:partition, [[228], [], [228]]]\n",
            "[:rpartition, [[228, 228], [], []]]\n",
            "[:lines, [[228, 228]]]\n",
            "[:strip, [\"ISO-8859-1\", [228, 228]]]\n",
            "[:tr, [\"ISO-8859-1\", [228, 228]]]\n",
            "[:delete, [\"ISO-8859-1\", [228, 228]]]\n",
            "[:squeeze, [\"ISO-8859-1\", [228, 228]]]\n",
            "[:ljust, [\"ISO-8859-1\", [228, 228]]]\n",
            "[:plus, [\"ISO-8859-1\", [228, 228, 228, 228]]]\n",
            "[:times, [\"ISO-8859-1\", [228, 228, 228, 228]]]\n",
            "[:upcase, [\"ISO-8859-1\", [196, 196]]]\n",
            "[:downcase, [\"ISO-8859-1\", [228, 228]]]\n",
            "[:capitalize, [\"ISO-8859-1\", [228, 228]]]\n",
            "[:swapcase, [\"ISO-8859-1\", [196, 196]]]\n",
            "[:reverse, [\"ISO-8859-1\", [228, 228]]]\n",
            "[:each_line, [[228], [228]]]\n",
            "[:match_capture, [\"ISO-8859-1\", [228]]]\n",
            "[:pre_match, [\"ISO-8859-1\", [228]]]\n",
            "[:post_match, [\"ISO-8859-1\", [228]]]\n",
            "[:dollar, [\"ISO-8859-1\", [228]]]\n",
            "[:format, [\"ISO-8859-1\", [228, 228]]]\n",
            "[:dup, [\"ISO-8859-1\", [228, 228]]]\n",
            "[:slice_bang, [\"ISO-8859-1\", [228, 228]]]\n",
            "[:insert, [\"ISO-8859-1\", [228, 228]]]\n",
            "[:concat, [\"ISO-8859-1\", [228, 228]]]\n",
            "[:unpack, [\"ASCII-8BIT\", [228, 228]]]\n",
            "[:each_char, [[], [], [], [], [], [], [], [], [], []]]\n",
            "[:byteslice, [\"ISO-8859-1\", [228]]]\n",
            "[:center, [\"ISO-8859-1\", [228, 228]]]\n",
            "[:chomp, [\"ISO-8859-1\", [228, 228]]]\n",
            "[:chop, [\"ISO-8859-1\", [228, 228]]]\n",
            "[:split_limit, [[228], [228]]]\n",
            "[:start_with, true]\n",
            "[:to_s, [\"ISO-8859-1\", [228, 228]]]\n",
            "[:to_str, [\"ISO-8859-1\", [228, 228]]]\n",
            "[:encode, [\"UTF-8\", [195, 164, 195, 164]]]\n",
            "[:freeze_dup, [\"ISO-8859-1\", [228, 228]]]\n",
            "[:interpolated, [\"ISO-8859-1\", [228, 228]]]\n",
            "2\n",
            "2\n",
            "[195, 169]\n",
            "[195]\n",
            "[169, 195]\n",
            "2\n",
            "[169, 195]\n",
        )
    );
}

#[test]
fn encoding_find_refuses_what_is_not_a_name() {
    let output = run_example("encodings/encoding_find_refuses_non_strings.rb");
    assert_eq!(
        output,
        concat!(
            "[TypeError, \"no implicit conversion of nil into String\"]\n",
            "[TypeError, \"no implicit conversion of Integer into String\"]\n",
            "[TypeError, \"no implicit conversion of Symbol into String\"]\n",
            "#<Encoding:UTF-8>\n",
        )
    );
}

#[test]
fn gsub_takes_the_encoding_of_a_non_ascii_replacement() {
    let output = run_example("encodings/gsub_takes_the_replacement_encoding.rb");
    assert_eq!(
        output,
        concat!(
            "\"måmmå let me go\"\n",
            "#<Encoding:UTF-8>\n",
            "#<Encoding:US-ASCII>\n",
            "#<Encoding:UTF-8>\n",
            "#<Encoding:UTF-8>\n",
            "#<Encoding:UTF-8>\n",
            "#<Encoding:UTF-8>\n",
        )
    );
}

#[test]
fn unicode_normalize_refuses_bytes_that_spell_nothing() {
    let output = run_example("encodings/unicode_normalize_refuses_broken_text.rb");
    assert_eq!(
        output,
        "false\n[ArgumentError, \"invalid byte sequence in UTF-8\"]\n\"ok\"\n\"é\"\n"
    );
}

#[test]
fn korean_double_byte_text_reads_and_writes() {
    let output = run_example("encodings/korean_double_byte_text_reads_and_writes.rb");
    assert_eq!(
        output,
        concat!(
            "\"스티해\"\n",
            "\"스티해\"\n",
            "[189, 186, 198, 188, 199, 216]\n",
            "\"콭\"\n",
            "2\n",
            "false\n",
            "3\n",
            "\"incomplete \\\"\\\\xB2\\\" on EUC-KR\"\n",
        )
    );
}

#[test]
fn gsub_takes_a_replacement_in_another_encoding() {
    let output = run_example("encodings/gsub_takes_a_replacement_in_another_encoding.rb");
    assert_eq!(
        output,
        concat!(
            "#<Encoding:UTF-16BE>\n",
            "[0, 104, 0, 105]\n",
            "#<Encoding::CompatibilityError: incompatible character encodings: UTF-8 and UTF-16BE>\n",
            "#<Encoding:UTF-16BE>\n",
            "[0, 104, 0, 105, 0, 104, 0, 105]\n",
            "#<Encoding:EUC-JP>\n",
            "[164, 162]\n",
            "#<Encoding:UTF-8>\n",
            "#<Encoding:UTF-8>\n",
        )
    );
}

#[test]
fn gb18030_text_reads_and_writes() {
    let output = run_example("encodings/gb18030_text_reads_and_writes.rb");
    assert_eq!(
        output,
        r#""開"
1
true
["80", "ffff", "10000", "10ffff"]
4
[233, 95, 32, 129, 48, 129, 48, 32, 132, 49, 164, 57, 32, 144, 48, 129, 48, 32, 227, 50, 154, 53, 32, 162, 227]
true
false
false
false
false
1
Encoding::UndefinedConversionError
"\"\\x841\\xA50\" from GB18030 to UTF-8"
[132, 49, 165, 48]
#<Encoding:GB18030>
Encoding::UndefinedConversionError
"\"\\xA2\\xA0\" from GBK to UTF-8"
"�z"
"#
    );
}

#[test]
fn utf16_conversion_refuses_a_lone_surrogate() {
    let output = run_example("encodings/utf16_conversion_refuses_a_lone_surrogate.rb");
    assert_eq!(
        output,
        r#"[154, 138, 222, 186, 155, 104, 247, 79, 128, 9]
Encoding::InvalidByteSequenceError
"\"\\xDE\\xBA\" on UTF-16BE"
[65536]
false
"�"
[Encoding::InvalidByteSequenceError, "\"\\xDC\\x00\" on UTF-16BE", [220, 0], [], false]
"a�b"
[Encoding::InvalidByteSequenceError, "\"\\xD8\\x00\" followed by \"\\x00\" on UTF-16BE", [216, 0], [0], false]
"a�b"
[Encoding::InvalidByteSequenceError, "incomplete \"\\xD8\\x00\" on UTF-16BE", [216, 0], [], true]
"a�"
[Encoding::InvalidByteSequenceError, "incomplete \"\\x00\" on UTF-16BE", [0], [], true]
"a�"
[Encoding::InvalidByteSequenceError, "\"\\x00\\xDC\" on UTF-16LE", [0, 220], [], false]
"a�"
[Encoding::InvalidByteSequenceError, "\"\\x00\\x11\\x00\\x00\" on UTF-32BE", [0, 17, 0, 0], [], false]
"a�"
[Encoding::InvalidByteSequenceError, "\"\\x00\\x00\\xD8\\x00\" on UTF-32BE", [0, 0, 216, 0], [], false]
"�"
[Encoding::InvalidByteSequenceError, "incomplete \"\\x00\\x00\\x00\" on UTF-32BE", [0, 0, 0], [], true]
"�"
"#
    );
}

#[test]
fn gsub_over_binary_text_takes_a_utf8_replacement() {
    let output = run_example("encodings/gsub_over_binary_text_takes_a_utf8_replacement.rb");
    assert_eq!(
        output,
        r#""&AOQ-&-&AOQ-&-&AOQ-"
#<Encoding:BINARY (ASCII-8BIT)>
"ä&ä&ä"
#<Encoding:UTF-8>
true
"aäb"
#<Encoding:UTF-8>
Encoding::CompatibilityError
[195, 164]
[195, 164]
[195, 164]
[195, 164]
"#
    );
}

#[test]
fn iso_2022_jp_replaces_bytes_it_cannot_read() {
    let output = run_example("encodings/iso_2022_jp_replaces_bytes_it_cannot_read.rb");
    assert_eq!(
        output,
        r#"[68, 105, 106, 138, 97, 116]
#<Encoding:UTF-8>
true
"Dij?at"
Encoding::InvalidByteSequenceError
"\"\\x8A\" on ISO-2022-JP"
"あa"
"?"
"a?bあ?"
"a\x8Ab" => [Encoding::InvalidByteSequenceError, "\"\\x8A\" on ISO-2022-JP", [138], nil] | "a?b"
"\e$B$\"\x8A\e(B" => [Encoding::InvalidByteSequenceError, "\"\\x8A\" on ISO-2022-JP", [138], nil] | "あ?"
"\e$B$" => [Encoding::InvalidByteSequenceError, "incomplete \"$\" on ISO-2022-JP", [36], nil] | "?"
"\e$B$\n\e(B" => [Encoding::InvalidByteSequenceError, "\"$\" followed by \"\\n\" on ISO-2022-JP", [36], [10]] | "??"
"\e$B\n\e(B" => [Encoding::InvalidByteSequenceError, "\"\\n\" on ISO-2022-JP", [10], nil] | "?"
"\e$B\u007F!\e(B" => [Encoding::InvalidByteSequenceError, "\"\\x7F\" on ISO-2022-JP", [127], nil] | "??"
"\e$B$\u007F\e(B" => [Encoding::InvalidByteSequenceError, "\"$\" followed by \"\\x7F\" on ISO-2022-JP", [36], [127]] | "??"
"a\eb" => [Encoding::InvalidByteSequenceError, "\"\\e\" followed by \"b\" on ISO-2022-JP", [27], [98]] | "a?b"
"a\e$Ab" => [Encoding::InvalidByteSequenceError, "\"\\e$\" followed by \"A\" on ISO-2022-JP", [27, 36], [65]] | "a?Ab"
"\u000Ea" => [Encoding::InvalidByteSequenceError, "\"\\x0E\" on ISO-2022-JP", [14], nil] | "?a"
"\u000Fa" => [Encoding::InvalidByteSequenceError, "\"\\x0F\" on ISO-2022-JP", [15], nil] | "?a"
"\e$B\e(B" => "" | ""
"\e$B$\"\e(Jx" => "あx" | "あx"
"\e" => [Encoding::InvalidByteSequenceError, "incomplete \"\\e\" on ISO-2022-JP", [27], nil] | "?"
"\e$" => [Encoding::InvalidByteSequenceError, "incomplete \"\\e$\" on ISO-2022-JP", [27, 36], nil] | "?"
"\e$B!\x80\e(B" => [Encoding::InvalidByteSequenceError, "\"!\" followed by \"\\x80\" on ISO-2022-JP", [33], [128]] | "??"
"\e$B\"/\e(B" => [Encoding::UndefinedConversionError, "\"\\xA2\\xAF\" to UTF-8 in conversion from ISO-2022-JP to stateless-ISO-2022-JP to EUC-JP to UTF-8", [162, 175], nil] | "?"
"\e$B$\"\e$B$\"\e(B" => "ああ" | "ああ"
["UTF-8", Encoding::UndefinedConversionError, "\"\\xA2\\xAF\" to UTF-8 in conversion from ISO-2022-JP to stateless-ISO-2022-JP to EUC-JP to UTF-8", "EUC-JP", "UTF-8", #<Encoding:EUC-JP>]
["UTF-8", Encoding::InvalidByteSequenceError, "\"\\x8A\" on ISO-2022-JP", "ISO-2022-JP", "stateless-ISO-2022-JP", nil]
["UTF-16LE", Encoding::UndefinedConversionError, "\"\\xA2\\xAF\" to UTF-8 in conversion from ISO-2022-JP to stateless-ISO-2022-JP to EUC-JP to UTF-8 to UTF-16LE", "EUC-JP", "UTF-8", #<Encoding:EUC-JP>]
["UTF-16LE", Encoding::InvalidByteSequenceError, "\"\\x8A\" on ISO-2022-JP", "ISO-2022-JP", "stateless-ISO-2022-JP", nil]
"#
    );
}

#[test]
fn binary_text_keeps_its_bytes_through_string_steps() {
    let output = run_example("encodings/binary_text_keeps_its_bytes_through_string_steps.rb");
    assert_eq!(
        output,
        r#"27
10
27
#<Encoding:BINARY (ASCII-8BIT)>
[15]
[15]
[[4, 10]]
17
#<Encoding:BINARY (ASCII-8BIT)>
19
15
15
15
15
[99, 105, 195, 171, 32]
[99, 105, 195, 171]
[99, 105, 195, 171]
[99, 105, 195, 171]
"#
    );
}

#[test]
fn class_eval_reads_an_encoding_comment() {
    let output = run_example("encodings/class_eval_reads_an_encoding_comment.rb");
    assert_eq!(output, "#<Encoding:ISO-8859-7>\n[99, 97, 102, 233]\n");
}

#[test]
fn euc_jp_replaces_whole_characters() {
    let output = run_example("encodings/euc_jp_replaces_whole_characters.rb");
    assert_eq!(
        output,
        "5\n\"# \u{fffd}\u{fffd}\\n\"\ntrue\ntrue\n\"# \u{fffd}\\n\"\n\"\\\"\\\\x8E\\\\xE0\\\" from EUC-JP to UTF-8\"\n"
    );
}

#[test]
fn test_encodings_escaped_bytes_in_frozen_binary_source_execution() {
    let output = run_example("encodings/escaped_bytes_in_frozen_binary_source.rb");
    assert_eq!(
        output,
        "#<Encoding:BINARY (ASCII-8BIT)>\ntrue\n[239, 187, 191]\nfalse\ntrue\n"
    );
}

#[test]
fn test_encodings_escaped_bytes_in_frozen_utf8_source_execution() {
    let output = run_example("encodings/escaped_bytes_in_frozen_utf8_source.rb");
    assert_eq!(output, "true\n#<Encoding:UTF-8>\nfalse\n");
}

#[test]
fn test_encodings_encode_to_binary_takes_ascii_alone_execution() {
    let output = run_example("encodings/encode_to_binary_takes_ascii_alone.rb");
    assert_eq!(
        output,
        concat!(
            "#<Encoding::UndefinedConversionError: U+3042 from UTF-8 to ASCII-8BIT>\n",
            "#<Encoding::UndefinedConversionError: U+00C3 to ASCII-8BIT in conversion from ISO-8859-1 to UTF-8 to ASCII-8BIT>\n",
            "#<Encoding:BINARY (ASCII-8BIT)>\n\"?\"\n[255]\n",
        )
    );
}
