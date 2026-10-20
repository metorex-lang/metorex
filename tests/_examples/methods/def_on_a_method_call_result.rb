# `def name.method` where `name` is a method rather than a local defines the
# singleton method on the object that method answers.
class Shipment
  def label_target
    @label_target ||= Object.new
  end

  def define_label
    def label_target.label = "crate"
    label_target.label
  end
end

shipment = Shipment.new
puts shipment.define_label
p shipment.label_target.singleton_class.public_method_defined?(:label)
p Shipment.method_defined?(:label)
