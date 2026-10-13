module ConstantSpecOuter
  module Helper
    def self.label = "found"
  end

  class Builder
    class << self
      def define_named(name)
        define_singleton_method(name) { Helper.label }
      end
    end
  end
end

module ConstantSpecFramework
  def self.configure = :configured
end

ConstantSpecAlias = ConstantSpecFramework

module ConstantSpecAlias
  def self.added = :added
end

module ConstantSpecHelpers
  def helped = :top_level
end

module ConstantSpecMixins
  module ConstantSpecHelpers
    def helped = :nested
  end

  class Widget
    include ::ConstantSpecHelpers
  end

  class Gadget
    extend ::ConstantSpecHelpers
  end
end

module ConstantSpecKinds
  def self.of(value)
    case value
    when Array then :array
    else :other
    end
  end
end

class ConstantSpecHost
  Array = :shadow
end

module ConstantSpecVisible
  Local = :outer_local

  def self.read_in_basic_object
    BasicObject.class_exec { [Class, Local] }
  end
end

RSpec.describe "Constant lookup" do
  it "reads constants where define_singleton_method's block was written" do
    ConstantSpecOuter::Builder.define_named(:via_class_self)
    expect(ConstantSpecOuter::Builder.via_class_self).to eq("found")
  end

  it "reopens a module through a constant naming it" do
    expect([ConstantSpecAlias.equal?(ConstantSpecFramework), ConstantSpecFramework.added]).to eq([true, :added])
  end

  it "mixes in the top-level module an include names with ::" do
    expect([ConstantSpecMixins::Widget.new.helped, ConstantSpecMixins::Gadget.helped]).to eq([:top_level, :top_level])
  end

  it "reads a when clause's constant in the called method's scope" do
    kind = ConstantSpecHost.class_exec { ConstantSpecKinds.of([1]) }
    expect(kind).to eq(:array)
  end

  it "reaches top-level constants from a block BasicObject.class_exec runs" do
    expect(ConstantSpecVisible.read_in_basic_object).to eq([Class, :outer_local])
  end
end
