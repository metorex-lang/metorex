# Line coverage counts a line once each time the code reaches it, however
# many statements are written on it, as ActionView's compiled templates put
# several on one line; a block's body on its call's line counts on its own.
require "coverage"
Coverage.start(lines: true)
load File.join(__dir__, "target.rb")
p Coverage.result[File.join(__dir__, "target.rb")][:lines]
