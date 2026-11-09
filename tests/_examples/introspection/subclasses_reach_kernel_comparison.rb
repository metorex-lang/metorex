reading = Class.new
p reading.instance_method(:<=>).owner
p reading.instance_method(:<=>).bind(reading.new).call(1)
