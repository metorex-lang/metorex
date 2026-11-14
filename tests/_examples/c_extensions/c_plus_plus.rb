# A C extension written in C++: create_makefile finds the .cc source and
# builds it with the C++ compiler, and `ruby -r` loads the extension it
# makes as `require` would.
require "fileutils"
require "tmpdir"

Dir.mktmpdir do |directory|
  FileUtils.cp(File.join(__dir__, "c_plus_plus.cc"), directory)
  configure = '$srcdir = Dir.pwd; create_makefile("c_plus_plus")'
  system(RbConfig.ruby, "-rmkmf", "-e", configure, chdir: directory, out: File::NULL) or raise "extconf failed"
  system("make", chdir: directory, out: File::NULL, err: File::NULL) or raise "make failed"
  system(RbConfig.ruby, "-I", directory, "-r", "c_plus_plus", "-e", 'puts Loud.shout("quiet please")')
end
