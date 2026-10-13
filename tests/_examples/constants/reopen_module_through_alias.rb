module Framework
  def self.configure = :configured
end

Alias = Framework

module Alias
  def self.added = :added
end

p [Alias.equal?(Framework), Framework.added, Alias.configure]
module Holder
  module Inner
    def self.where = :holder
  end
end
include Holder
module Inner
  def self.top = :top
end
p [Inner.equal?(Holder::Inner), Inner.respond_to?(:top)]
