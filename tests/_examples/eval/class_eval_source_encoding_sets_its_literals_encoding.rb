# Source handed to class_eval is read in the encoding its String carries, or
# the one a magic comment on its first line names, which is the encoding of
# the String literals written in it, as Tilt's compiled templates rely on.
shift_jis = "def shift_jis_literal = \"ふが\"\n".encode("Shift_JIS")
Object.class_eval(shift_jis)
p [shift_jis_literal.encoding, shift_jis_literal.encode("UTF-8")]

commented = "# coding: Shift_JIS\ndef commented_literal = \"ふが\"\n".encode("Shift_JIS").force_encoding("UTF-8")
Object.class_eval(commented)
p [commented_literal.encoding, commented_literal.encode("UTF-8")]

ascii = "def ascii_literal\n# coding: UTF-8\n'x'\nend\n".dup.force_encoding("US-ASCII")
Object.class_eval(ascii)
p ascii_literal.encoding

p [eval("'x'".encode("Shift_JIS")).encoding, eval("# coding: EUC-JP\n'x'").encoding]
