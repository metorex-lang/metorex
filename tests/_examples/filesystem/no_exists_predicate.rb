# File.exists? and Dir.exists? were removed in Ruby 3.2, so neither class
# answers them, and asking for either as a method names the class.
p File.respond_to?(:exists?), Dir.respond_to?(:exists?)
[File, Dir].each do |klass|
  begin
    klass.exists?("/")
  rescue NoMethodError => error
    puts error.message
  end
  begin
    klass.method(:exists?)
  rescue NameError => error
    puts error.message
  end
end
