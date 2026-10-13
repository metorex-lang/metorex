module MethodSpecs
  class Doubler
    def each_doubled(transform = proc { |value| yield value * 2 })
      [1, 2, 3].map { |value| transform.call(value) }
    end

    def first_given(value = yield)
      value
    end

    def block_given_in_default(given = block_given?, &block)
      given
    end
  end

  class Includer
    def include(item)
      [:included, item]
    end

    def run
      first = include(1).first
      from_brace_block = proc { include(2).last }.call
      from_lambda = lambda { include(3).size }.call
      [first, from_brace_block, from_lambda]
    end
  end

  class BlankSlate
    kept = [:__id__, :__send__, :object_id, :class, :respond_to?]
    instance_methods.each do |name|
      undef_method(name) unless kept.include?(name)
    end
  end

  class Missing
    undef_method :<=>
    undef_method :==
    undef_method :inspect

    def method_missing(name, *arguments)
      [:missing, name, *arguments]
    end

    def respond_to_missing?(*)
      true
    end
  end

  class OrderedWithoutComparable
    def <=>(_other)
      0
    end

    def method_missing(name, *arguments)
      [:missing, name, *arguments]
    end

    def respond_to_missing?(*)
      true
    end
  end

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

  module Settings
    def level=(value)
      @level = value
    end
  end

  class Widget
    def size=(value)
      @size = value
    end
  end
end

RSpec.describe "A method" do
  let(:doubler) { MethodSpecs::Doubler.new }

  it "yields to its block from a default parameter value" do
    expect(doubler.each_doubled { |value| value + 1 }).to eq([3, 5, 7])
  end

  it "uses a passed argument over a default that yields" do
    expect(doubler.each_doubled(proc { |value| value })).to eq([1, 2, 3])
  end

  it "takes a default value from yield" do
    expect(doubler.first_given { :from_the_block }).to eq(:from_the_block)
  end

  it "reads block_given? in a default parameter value" do
    expect([doubler.block_given_in_default {}, doubler.block_given_in_default]).to eq([true, false])
  end

  it "calls a method named include with a receiverless call chained on it" do
    expect(MethodSpecs::Includer.new.run).to eq([:included, 2, 2])
  end
end

RSpec.describe "undef_method" do
  it "removes the methods a class inherits from Object and Kernel" do
    expect(MethodSpecs::BlankSlate.instance_methods & [:<=>, :inspect, :instance_eval, :public_methods]).to eq([])
  end

  it "keeps the methods it was not asked to remove" do
    expect(MethodSpecs::BlankSlate.instance_methods).to include(:__id__, :__send__, :class, :object_id, :respond_to?)
  end

  it "sends an operator call to method_missing" do
    expect(MethodSpecs::Missing.new <=> 1).to eq([:missing, :<=>, 1])
  end

  it "sends an equality call to method_missing" do
    expect(MethodSpecs::Missing.new == 2).to eq([:missing, :==, 2])
  end

  it "sends a call of a native method to method_missing" do
    expect(MethodSpecs::Missing.new.inspect).to eq([:missing, :inspect])
  end
end

RSpec.describe "A comparison operator on an object that is not Comparable" do
  it "is sent to method_missing" do
    expect(MethodSpecs::OrderedWithoutComparable.new < 5).to eq([:missing, :<, 5])
  end

  it "leaves equality to BasicObject even with <=> defined" do
    expect(MethodSpecs::OrderedWithoutComparable.instance_method(:==).owner).to eq(BasicObject)
  end
end

RSpec.describe "respond_to? on a module" do
  it "does not ask the module's own instance method respond_to_missing?" do
    expect(MethodSpecs::Tools.respond_to?(:find_all)).to be false
  end

  it "answers through respond_to_missing? for an instance" do
    expect(MethodSpecs::Searcher.new.respond_to?(:find_all)).to be true
  end

  it "reports Module's private ruby2_keywords when asked with private methods included" do
    expect(MethodSpecs::Tools.respond_to?(:ruby2_keywords, true)).to be true
  end
end

RSpec.describe "An attribute assignment to a class or module" do
  it "raises NoMethodError for a module with only an instance writer" do
    expect { MethodSpecs::Settings.level = 1 }.to raise_error(NoMethodError, /undefined method 'level='/)
  end

  it "raises NoMethodError for a class with only an instance writer" do
    expect { MethodSpecs::Widget.size = 2 }.to raise_error(NoMethodError, /undefined method 'size='/)
  end

  it "stores no class variable" do
    begin
      MethodSpecs::Widget.color = 3
    rescue NoMethodError
      nil
    end
    expect(MethodSpecs::Widget.class_variables).to eq([])
  end
end

RSpec.describe "Kernel functions on a class" do
  let(:puts_method) { String.method(:puts) }

  it "are handed out as methods Kernel owns" do
    expect([puts_method.name, puts_method.owner]).to eq([:puts, Kernel])
  end

  it "are not public methods" do
    expect { String.public_method(:puts) }.to raise_error(NameError)
  end
end

RSpec.describe "instance_method on a class of the program's own" do
  let(:klass) { Class.new }

  it "hands out a Kernel method Kernel owns" do
    expect(klass.instance_method(:inspect).owner).to eq(Kernel)
  end

  it "hands out a BasicObject method BasicObject owns" do
    expect(klass.instance_method(:instance_eval).owner).to eq(BasicObject)
  end

  it "names an anonymous class in the error for a missing method" do
    expect { klass.instance_method(:nope) }.to raise_error(NameError, /for class '#<Class:0x\h+>'/)
  end
end
