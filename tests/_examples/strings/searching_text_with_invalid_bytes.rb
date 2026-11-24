broken = "こにちわ\n".dup
broken << "\xB8\x9E\b\x88\xA5\n".b.force_encoding("UTF-8")
p broken.valid_encoding?
p broken.include?("こにちわ")
p broken.index("にち")
p broken.start_with?("こに")
p broken.include?("\x88".b.force_encoding("UTF-8"))
mixed = "こにちわ".dup
mixed << "\xff"
p mixed.include?("ちわ")
p mixed.end_with?("\xff")
u = ->(text) { text.b.force_encoding("UTF-8") }
p u["a\xffb"].include?(u["\xff"])
p u["\xB8\x9E\b\x88\xA5"].include?(u["\x88"])
p u["\b\x88\xA5"].include?(u["\x88"])
p u["\x88\xA5"].include?(u["\x88"])
p u["\x88"].include?(u["\x88"])
p u["a\x88"].include?(u["\x88"])
p u["\b\x88"].include?(u["\x88"])
p u["\x9E\b\x88"].include?(u["\x88"])
p u["x\x88"].index(u["\x88"])
