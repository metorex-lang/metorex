module Tool
  class BasicObject; end

  def self.capture
    TOPLEVEL_BINDING.eval("def self.__capture__\n binding\nend\n$captured = __capture__")
  end

  def self.run(code) = $captured.eval(code)
end
Tool.capture
p Tool.run("BasicObject")
