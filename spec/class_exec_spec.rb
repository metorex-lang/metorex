module ClassExecSpecs
  class Group
    def self.context(&block)
      Class.new(self).class_exec(&block)
    end

    def self.with_wrapper(&block)
      context do
        module_exec(&block)
      end
    end
  end

  def self.run_in(klass)
    value = :caller_local
    seen = klass.class_exec(:argument) { |value| value }
    [seen, value]
  end
end

RSpec.describe "class_exec" do
  it "binds a splat parameter with no arguments" do
    expect(Class.new.class_exec { |*options| options }).to eq([])
  end

  it "binds a splat parameter with arguments" do
    expect(Class.new.class_exec(skip: true) { |*options| options }).to eq([{ skip: true }])
  end

  it "reads a captured local named like a Kernel function" do
    names = [:be_a].map do |method|
      Class.new.class_exec { method }
    end
    expect(names).to eq([:be_a])
  end

  it "reads its captured block over a caller parameter of the same name" do
    parent = ClassExecSpecs::Group.with_wrapper { superclass }
    expect(parent).to eq(ClassExecSpecs::Group)
  end

  it "binds its parameters without changing the caller's locals" do
    expect(ClassExecSpecs.run_in(Class.new)).to eq([:argument, :caller_local])
  end
end
