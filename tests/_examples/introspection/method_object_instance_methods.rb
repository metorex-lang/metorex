# UnboundMethod's own bind and bind_call, and GC's garbage_collect, found
# with instance_method as the other methods of their modules are.
p UnboundMethod.instance_method(:bind).parameters
p UnboundMethod.instance_method(:bind_call).parameters
p GC.instance_method(:garbage_collect).parameters
p UnboundMethod.instance_method(:bind).owner
p GC.instance_method(:garbage_collect).owner
