module Diff
  module LCS
    def self.name_here = "top-level Diff::LCS"
  end
end

module Library
  module Diff
  end

  class Base
  end

  module Helpers
  end
end

class Spec < Library::Base
  include Library::Helpers
end

Spec.class_exec do
  p Diff::LCS.name_here
end

Class.new(Library::Base).class_exec do
  p Diff::LCS.name_here
end

module Library
  class Base
    def self.make(&block)
      Class.new(self).tap { |made| made.module_exec(&block) }
    end
  end
end

Library::Base.make do
  p Diff::LCS.name_here
end
