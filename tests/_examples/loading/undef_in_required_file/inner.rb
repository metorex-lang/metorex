def helper_path(path) = "root/#{path}"
module Tools
  NAME = helper_path("x")
end
undef helper_path
p self, Module.nesting
