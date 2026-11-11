# frozen_string_literal: true

p "\xAA".bytes
p (+"\xAA").bytes
p (+"\xAA").force_encoding(Encoding::ISO_8859_1).encode(Encoding::UTF_8).bytes
p (+"café").bytes
