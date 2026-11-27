class Recorder
  def self.declare(name, &block)
    define_method(name, &block)
  end

  declare :first do
    :ran

    :again
  end

  declare(:second) { :ran }

  declare :third do |value|
    value
  end
end

%i[first second third].each do |name|
  p Recorder.instance_method(name).source_location.last
end
