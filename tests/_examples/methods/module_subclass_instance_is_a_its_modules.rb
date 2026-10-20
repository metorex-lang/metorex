# A module made by a subclass of Module is an instance of that subclass and
# of the modules it includes, which `is_a?`, `===`, and a `case` all report.
module Doubling; end

class ModuleDouble < Module
  include Doubling
end

double = ModuleDouble.new
p double.is_a?(Doubling)
p Doubling === double
p ModuleDouble === double
p double.kind_of?(Module)
p(case double when Doubling then "doubling" else "other" end)
