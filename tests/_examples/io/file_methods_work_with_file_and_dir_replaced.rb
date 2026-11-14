# A library that swaps the File and Dir constants for classes of its own, as
# FakeFS does, still reaches the original File's path methods through a
# constant it saved.
RealFile = File
RealDir = Dir
module Fake
  class File
    def self.expand_path(*) = "fake"
  end

  class Dir
    def self.pwd = "/fake"
  end
end
Object.send(:remove_const, :File)
Object.const_set(:File, Fake::File)
Object.send(:remove_const, :Dir)
Object.const_set(:Dir, Fake::Dir)
begin
  p RealFile.expand_path("/a/b/../c")
  p RealFile.expand_path("x") == RealFile.join(RealDir.pwd, "x")
  p RealFile.absolute_path("/a/./b")
  p RealFile.absolute_path?("/a")
  p RealFile.join("a", "b", ["c", "d"])
  p RealFile.expand_path("~") == ENV["HOME"]
  p File.expand_path("anything")
ensure
  Object.send(:remove_const, :File)
  Object.const_set(:File, RealFile)
  Object.send(:remove_const, :Dir)
  Object.const_set(:Dir, RealDir)
end
p File.expand_path("/a/../b")
