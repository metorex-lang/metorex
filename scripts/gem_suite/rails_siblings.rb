# Links the rails repository's other directories beside the copy of one of
# its frameworks a test file runs in, since the tests' helpers reach the
# repository's tools and the other frameworks by their place in it.
require "fileutils"

repository = File.expand_path("../../gems/rails", __dir__)
framework = File.basename(Dir.pwd)
Dir.children(repository).each do |sibling|
  next if sibling == framework || sibling.start_with?(".")

  FileUtils.ln_s(File.join(repository, sibling), File.join("..", sibling))
end
