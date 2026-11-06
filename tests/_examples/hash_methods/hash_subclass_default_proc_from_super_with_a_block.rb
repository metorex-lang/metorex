# A Hash subclass whose initialize hands a block to `super()` reads missing
# Symbol keys through that block, and `to_h` through an alias keeps the
# entries.
class Params < Hash
  alias_method :to_params_hash, :to_h
end

Keyed = Class.new(Params) do
  def initialize(*)
    super() { |hash, key| hash[key.to_s] if key.is_a?(Symbol) }
  end
end

params = Keyed.new
params["foo"] = "bar"
p params[:foo]
p params[:missing]
p params.default_proc.nil?
nested = Keyed.new
nested["inner"] = Keyed.new
nested["inner"]["type"] = "text/plain"
p nested[:inner][:type]
p params.to_params_hash
p params.to_params_hash.class
p params.to_params_hash[:foo]
p params.to_h.default_proc.nil?
plain = Hash.new { |hash, key| key.to_s }
p plain.to_h[:kept]
