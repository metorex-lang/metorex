# File.open takes the new file's permissions as perm: the way it takes them
# as a third argument.
require "tmpdir"

Dir.mktmpdir do |directory|
  by_keyword = File.join(directory, "by_keyword")
  File.open(by_keyword, File::RDWR | File::CREAT | File::EXCL, perm: 0600) { |file| file.write("kept") }
  printf("%o\n", File.stat(by_keyword).mode & 0777)

  by_argument = File.join(directory, "by_argument")
  File.open(by_argument, File::RDWR | File::CREAT | File::EXCL, 0640) { |file| file.write("kept") }
  printf("%o\n", File.stat(by_argument).mode & 0777)
end
