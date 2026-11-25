location = eval("caller_locations(0, 1).first", binding, "(template)", 1)
p location.path
RubyVM::AbstractSyntaxTree.node_id_for_backtrace_location(location)
p :answered
