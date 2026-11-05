# A default proc's answer is what a read of a missing key answers, and a
# Hash subclass hands its default proc the instance itself. A method that
# answers the hash answers the instance.
counted = Hash.new { |hash, key| hash[key] = 1; nil }
p counted[:a], counted

class TextKeys < Hash
  def []=(key, value)
    super(key.to_s, value)
  end
end

stored = TextKeys.new { |hash, key| hash[key] = 5; nil }
p stored[:z], stored

class Plain < Hash
end

plain = Plain.new
plain[1] = 2
p plain.transform_values! { |value| value }.class
p plain.deconstruct_keys([]).class
p plain.keep_if { true }.class
p plain.select! { true }.class
