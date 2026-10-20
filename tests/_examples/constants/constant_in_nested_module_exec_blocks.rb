# A constant assigned in a block that `module_exec` runs, inside another such
# block, belongs to the scope the outermost block was written in.
def group(&block)
  Class.new.module_exec(&block)
end

group do
  group do
    DummyClass = Class.new do
      def foo = 3
    end
  end
end

p Object.const_defined?(:DummyClass, false)
p Object.const_get("DummyClass").new.foo
