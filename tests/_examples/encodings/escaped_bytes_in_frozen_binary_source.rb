# encoding: ascii-8bit
# frozen_string_literal: true
# Frozen literals escaping bytes in a BINARY source stay BINARY, even when
# the bytes spell UTF-8, so a binary String can be checked for a byte order
# mark, as parser's Source::Buffer does.
bom = "\xef\xbb\xbf"
p bom.encoding, bom.frozen?, bom.bytes
p "caf\xE9".b.start_with?(bom), "\xef\xbb\xbfx".b.start_with?(bom)
