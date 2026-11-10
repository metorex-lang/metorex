# A frozen_string_literal magic comment after code is ignored, and $VERBOSE
# says so with the file and line, while a heredoc line or a comment that only
# mentions the words is no magic comment, as RuboCop's generator templates
# hold one.
require "tmpdir"

directory = Dir.mktmpdir
sources = {
  "after_code.rb" => "entry = 1\n# frozen_string_literal: true\n",
  "mentioned.rb" => "entry = 1\n# the frozen_string_literal comment goes first\n",
  "in_heredoc.rb" => "template = <<~RUBY\n  # frozen_string_literal: true\nRUBY\n",
  "emacs_style.rb" => "entry = 1\n# -*- frozen_string_literal: true -*-\n"
}
warnings = []
$stderr.define_singleton_method(:write) { |*text| warnings << text.join; text.join.size }
$VERBOSE = true
sources.each do |name, source|
  path = File.join(directory, name)
  File.write(path, source)
  load path
end
eval("entry = 1\n# frozen_string_literal: true\n", nil, "evaluated.rb", 10)
$VERBOSE = false
$stderr.singleton_class.remove_method(:write)
puts warnings.grep(/frozen_string_literal/).map { |line| line.sub(directory + "/", "") }
