class Named < Module
  def initialize(label, &body)
    @label = label
    module_exec(&body) if body
  end

  attr_reader :label

  def inspect = "#<Named #{@label}>"

  def included(base)
    base.instance_variable_set(:@included_by, @label)
  end
end

shared = Named.new("logger") { def logs = :logging }
host = Class.new
p host < shared
host.include(shared)
p host < shared
p host.new.logs
p host.instance_variable_get(:@included_by)
p [shared, shared.label, shared.class, shared.is_a?(Named), shared.is_a?(Module)]
plain = Named.new("plain")
p plain.instance_methods
p [shared.respond_to?(:label), shared.respond_to?(:included), plain.respond_to?(:logs)]
