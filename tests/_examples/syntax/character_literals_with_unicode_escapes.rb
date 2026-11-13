# A character literal may be written as a Unicode escape, as Asciidoctor
# writes its control characters.
CAN = ?\u0018
DEL = ?\u007f
p [CAN, DEL, ?\u{41}, ?\t, ?a]
