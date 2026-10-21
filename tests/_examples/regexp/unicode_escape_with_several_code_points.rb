listed = /[\u{0041 0130 1d7bb}x-z]/
p "A".match?(listed), "İ".match?(listed), "\u{1d7bb}".match?(listed), "y".match?(listed), "B".match?(listed)
p "abİc".gsub(/[\u{0130 0062}]/, "İ" => "i̇", "b" => "B")
p "\u{41 42 43}", "\u{1F600 20 41}"
p /\u{41 42}/.match?("AB"), /\u{41 42}/.source
p "AB".match(/\u{41 42}/)[0], "ABB".match(/\u{41 42}+/)[0], "x\\u{41 42}".match?(/\\u\{41 42\}/)
