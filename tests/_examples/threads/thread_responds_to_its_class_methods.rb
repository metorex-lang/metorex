# Thread answers respond_to? for each of its own class methods.
p Thread.respond_to?(:report_on_exception)
p Thread.respond_to?(:report_on_exception=)
p Thread.respond_to?(:abort_on_exception)
p Thread.respond_to?(:handle_interrupt)
p Thread.respond_to?(:shipping_label)
