class Group
  def self.context(&block)
    Class.new(self).class_exec(&block)
  end

  def self.with_wrapper(&block)
    context do
      module_exec(&block)
    end
  end

  with_wrapper { p [:ran, superclass.superclass] }
end

def run_in(klass, &block)
  value = :caller_local
  klass.class_exec(:argument) { |value| p value }
  p value
end
run_in(Class.new)
