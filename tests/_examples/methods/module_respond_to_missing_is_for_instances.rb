module Finder
  def respond_to_missing?(name, include_private = false)
    name.to_s.start_with?("find_") || super
  end
end

module Tools
  include Finder
end

class Searcher
  include Finder
end

p Finder.respond_to?(:find_all)
p Finder.respond_to?(:other, true)
p Tools.respond_to?(:find_all)
p Tools.respond_to?(:ruby2_keywords, true)
p Searcher.new.respond_to?(:find_all)
p Searcher.new.respond_to?(:other)
