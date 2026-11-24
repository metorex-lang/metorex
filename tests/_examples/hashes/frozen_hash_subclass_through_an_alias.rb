class IndifferentHash < Hash
  alias_method :regular_writer, :[]= unless method_defined?(:regular_writer)

  def []=(key, value)
    regular_writer(key.to_s, value)
  end
end

held = IndifferentHash.new
held[:proposed] = 0
held.freeze
p held.frozen?
[-> { held[:bad] = 40 }, -> { held.store("bad", 1) }, -> { held.regular_writer("x", 1) }].each do |write|
  write.call
rescue => error
  p [error.class, error.message.sub(/:.*/, "")]
end
p held
