# A subclass of File opens a path the way File does, as an instance of
# itself.
require "tmpdir"

class LoggedFile < File
end

Dir.mktmpdir do |directory|
  path = File.join(directory, "kept")
  file = LoggedFile.new(path, "wb+")
  p file.class, file.path == path
  file.write("hi")
  file.close
  anonymous = Class.new(File) { alias_method :to_path, :path unless method_defined?(:to_path) }.open(File.join(directory, "other"), "wb+")
  p anonymous.class.superclass
  anonymous.close
  LoggedFile.open(path) { |held| p held.class, held.read }
end
