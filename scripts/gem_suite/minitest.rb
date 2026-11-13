# Runs the test files named on the command line, as minitest's `minitest`
# command does, which requires each as "./" and its path, so a test reads its
# own __FILE__ as an absolute path.
require "minitest/sprint"

Minitest::Sprint.run
