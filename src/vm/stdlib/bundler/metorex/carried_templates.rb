# Added by metorex's scripts/vendor_bundler.rb: Bundler reads its templates
# from a directory beside its own files, and metorex carries them inside the
# interpreter, so they are written out once to a directory of the user's own
# and read from there.
require "fileutils"
require "tmpdir"

module Bundler
  module CarriedTemplates
    def self.path(relative)
      File.join(root, relative)
    end

    def self.root
      @root ||= begin
        root = File.join(Dir.tmpdir, "metorex-#{Process.uid}", "bundler-4.0.20")
        unless File.directory?(root)
          writing = "#{root}.#{Process.pid}"
          __carried_gem_data__("bundler").each do |path, text|
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
