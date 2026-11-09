class LessBasic < BasicObject
  def reading = 5
end
held = LessBasic.new
singleton = (class << held; self; end)
p ObjectSpace.each_object(singleton).count
p ObjectSpace.each_object(LessBasic).count
