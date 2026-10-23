class TracedError < RuntimeError
  def set_backtrace(lines)
    super
    @handed = lines.class
  end
end

begin
  raise TracedError, "boom"
rescue TracedError => e
  p e.instance_variable_get(:@handed)
  p e.backtrace.first.end_with?("in '<main>'")
end

begin
  raise TracedError, "named", ["given.rb:1"]
rescue TracedError => e
  p e.backtrace
end

held = TracedError.new("held")
held.set_backtrace(["first.rb:2"])
held.instance_variable_set(:@handed, nil)
begin
  raise held
rescue TracedError => e
  p e.instance_variable_get(:@handed)
  p e.backtrace
end
