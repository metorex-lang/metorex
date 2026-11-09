class Inquirer < String
  def method_missing(name, *args)
    name.end_with?("?") ? self == name[0..-2] : super
  end

  def respond_to_missing?(name, include_private = false)
    name.end_with?("?") || super
  end
end
environment = Inquirer.new("test")
p environment.to_sym
p [:default, environment].map!(&:to_sym)
