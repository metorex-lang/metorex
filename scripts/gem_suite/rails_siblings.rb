# Links the rails repository's other directories beside the copy of one of
# its frameworks a test file runs in, since the tests' helpers reach the
# repository's tools and the other frameworks by their place in it.
#
# When the suite's bundle takes the frameworks from this copy, as railties'
# does through GEM_SUITE_CHECKOUT, each framework is copied in place of a
# link, since Ruby loads a file under a link from the link's target. On macOS
# `cp -c` clones the files, which copies no data.
require "fileutils"
require "rbconfig"

repository = File.expand_path("../../gems/rails", __dir__)
framework = File.basename(Dir.pwd)
copy_frameworks = ENV.key?("GEM_SUITE_CHECKOUT")
clone = RbConfig::CONFIG["host_os"].include?("darwin") ? ["cp", "-cR"] : ["cp", "-R", "--reflink=auto"]
Dir.children(repository).each do |sibling|
  next if sibling == framework || sibling.start_with?(".")

  source = File.join(repository, sibling)
  target = File.join("..", sibling)
  if copy_frameworks && File.exist?(File.join(source, "#{sibling}.gemspec"))
    system(*clone, source, target, exception: true)
  else
    FileUtils.ln_s(source, target)
  end
end
