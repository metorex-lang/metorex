module Loader
  def register(name, path)
    autoload name, path
  end
end
dept = Module.new { extend Loader }
dept.register(:FooCop, File.join(__dir__, "foo_cop.rb"))
p dept.autoload?(:FooCop)&.end_with?("foo_cop.rb"), Loader.autoload?(:FooCop), Object.autoload?(:FooCop)
