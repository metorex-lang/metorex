require "nkf"

greeting = "こんにちは、世界".encode("Shift_JIS")
p greeting.bytesize
p NKF.guess(greeting)
p NKF.nkf("-w", greeting)
p NKF.nkf("-w --hiragana", "カタカナ")
