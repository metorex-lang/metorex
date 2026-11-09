Plain = Class.new(BasicObject) do
  def label = :plain
end
Other = Class.new(BasicObject)
p Plain.instance_method(:label).bind(Plain.new).call
begin
  Plain.instance_method(:label).bind(Other.new)
rescue TypeError => error
  p error.message
end
