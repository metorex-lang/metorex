body = proc do |*options|
  options = options.first || {}

  define_method(:first) { :ran } unless options[:skip]
  p options
end

Class.new.class_exec(&body)
Class.new.class_exec({ skip: true }, &body)
