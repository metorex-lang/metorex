# `bundle init` writes a new Gemfile from the template Bundler carries.
require "bundler"
require "bundler/cli"
require "stringio"
require "tmpdir"

Dir.mktmpdir do |directory|
  Dir.chdir(directory) do
    said = StringIO.new
    $stdout = said
    Bundler::CLI.start(["init"])
    $stdout = STDOUT
    puts said.string.sub(File.realpath(directory), "DIRECTORY")
    puts File.read("Gemfile")
  end
end
