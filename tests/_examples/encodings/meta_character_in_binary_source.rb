# encoding: binary
# A meta character literal in a source written in bytes is a byte too.
p ?\M-z.encoding, ?\M-z == "\xFA", ?\M-\C-z.bytes, ?\C-z.encoding
