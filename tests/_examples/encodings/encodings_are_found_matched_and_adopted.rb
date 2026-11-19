# Finding an encoding takes any Encoding, and case equality on the Encoding
# class answers true for each one. An ASCII binary string that takes UTF-8
# text reads as that text, and an empty string takes UTF-16 text's encoding.
p [Encoding::UTF_8, Encoding::Shift_JIS, Encoding::ISO_8859_1].map { |held| Encoding.find(held).name }
p [Encoding === Encoding::Shift_JIS, Object === Encoding::UTF_8, Kernel === Encoding::UTF_8, String === Encoding::UTF_8]
chosen = case Encoding::ISO_8859_1
         when Encoding then :encoding
         else :other
         end
p chosen

text = "ab".b
text << "nyú"
p [text, text.encoding, text.valid_encoding?, text.bytes.last(3)]

wide = "Same".encode("UTF-16LE")
gathered = +""
gathered << wide
p [gathered.encoding, gathered == wide]
