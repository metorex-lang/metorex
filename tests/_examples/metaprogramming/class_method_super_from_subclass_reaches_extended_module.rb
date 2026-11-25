module Fixtures
  def fixtures(*names) = [:module, names]
end

class RecordCase
  extend Fixtures

  def self.fixtures(*names)
    super
  end
end

class TopicsCase < RecordCase
  p fixtures(:topics)
end

module Prefixes
  def local_prefixes = ["base"]
end

class Controller
  extend Prefixes

  def self.local_prefixes
    super + ["me3"]
  end

  class Inheriting < self; end
end

p Controller::Inheriting.local_prefixes
