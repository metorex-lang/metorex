p Time.methods.include?(:now), Time.methods.include?(:at)
p File.methods.include?(:exist?), File.methods.include?(:read), Dir.methods.include?(:pwd)
p Process.methods.include?(:clock_gettime), Integer.methods.include?(:sqrt)
p Time.methods.include?(:new), Time.methods.include?(:allocate), Time.methods.include?(:puts)
class << Time
  undef_method :at
end
p Time.methods.include?(:at), Time.respond_to?(:at)
