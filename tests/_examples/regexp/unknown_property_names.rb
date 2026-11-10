# A `\p{...}` naming no property Onigmo knows is refused, `newline` among
# them, while the names it knows read without regard to case and spacing.
p((Regexp.new("\\p{^Foo}") rescue $!))
p((Regexp.new("\\P{Foo Bar}") rescue $!))
p((Regexp.new("\\p{newline}") rescue $!))
p Regexp.new("\\p{ Alpha }"), "ぁ" =~ /\p{Hiragana}/, "a" =~ /[\p{L}&&\p{^Lu}]/
