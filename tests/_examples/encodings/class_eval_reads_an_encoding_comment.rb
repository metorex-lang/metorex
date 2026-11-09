source = "# encoding: ISO-8859-7\nGREETING = \"caf\xE9\"\n".b.force_encoding(Encoding::UTF_8)
Object.class_eval(source, "greeting.rb")
p GREETING.encoding
p GREETING.bytes
