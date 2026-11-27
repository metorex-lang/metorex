# A String subclass appended to an empty binary String gives it the
# subclass's encoding, as a plain String does.
class Fragment < String; end

body = String.new
body << Fragment.new("café ✓")
p body.encoding
p body

ascii = String.new
ascii << Fragment.new("plain")
p ascii.encoding

joined = String.new.concat(Fragment.new("✓"), "x")
p joined.encoding

# One built from another, or replacing a String's text, keeps its encoding.
p Fragment.new(Fragment.new("café")).encoding
replaced = "x".b
replaced.replace(Fragment.new("café"))
p replaced.encoding
