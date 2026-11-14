# Links the aws-sdk-ruby repository's other gems and its build tools beside
# the copy of one of its gems a spec file runs in, since the specs' helpers
# put those on the load path by their place in the repository.
require "fileutils"

repository = File.expand_path("../../gems/aws-sdk-ruby", __dir__)
gem_name = File.basename(Dir.pwd)
Dir.children(File.join(repository, "gems")).each do |sibling|
  next if sibling == gem_name

  FileUtils.ln_s(File.join(repository, "gems", sibling), File.join("..", sibling))
end
FileUtils.ln_s(File.join(repository, "build_tools"), File.join("..", "..", "build_tools"))
