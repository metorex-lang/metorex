# Added by metorex's scripts/vendor_gems.rb: rbs reads the signatures of
# Ruby's core and standard library from directories beside its own, and
# metorex carries them inside the interpreter, so they are written out once
# to a directory of the user's own and read from there.
require "fileutils"
require "tmpdir"

module RBS
  module CarriedData
    def self.root
      @root ||= begin
        root = File.join(Dir.tmpdir, "metorex-#{Process.uid}", "rbs-3.10.0")
        unless File.directory?(root)
          writing = "#{root}.#{Process.pid}"
          __carried_gem_data__("rbs").each do |path, text|
            file = File.join(writing, path)
            FileUtils.mkdir_p(File.dirname(file))
            File.binwrite(file, text)
          end
          begin
            File.rename(writing, root)
          rescue SystemCallError
            # Another process wrote the same files first.
            FileUtils.rm_rf(writing)
          end
        end
        root
      end
    end
  end
end
