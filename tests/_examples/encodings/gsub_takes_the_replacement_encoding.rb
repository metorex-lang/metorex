ascii = "%{who} let me go".dup.force_encoding(Encoding::US_ASCII)
result = ascii.gsub(/%\{who\}/) { "måmmå" }
p result, result.encoding
p ascii.gsub(/%\{who\}/) { "plain" }.encoding
p ascii.gsub(/%\{who\}/, "måmmå").encoding, ascii.sub(/%\{who\}/) { "å" }.encoding
p ascii.gsub("let", "låt").encoding
binary = "a-b".b
p binary.gsub("-") { "é" }.encoding rescue p $!.class
