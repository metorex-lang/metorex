# Time.new with no date reads the clock itself rather than calling Time.now,
# so a replacement for Time.now that takes no arguments is not handed any.
Time.singleton_class.alias_method :__real_now, :now
Time.define_singleton_method(:now) { "stubbed" }
p Time.new.class
p Time.new(in: "UTC").utc?
p Time.now
Time.singleton_class.alias_method :now, :__real_now
p Time.now.class
