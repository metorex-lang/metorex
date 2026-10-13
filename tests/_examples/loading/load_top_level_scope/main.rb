module Diff
end

module Library
  module Diff
  end

  module Loader
    def self.run(path)
      load path
    end
  end
end

Library::Loader.run(File.join(__dir__, "loaded.rb"))
