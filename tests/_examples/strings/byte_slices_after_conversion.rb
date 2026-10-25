# `byteslice` reads the string once its arguments are converted, so an
# argument whose `to_int` changes the string cuts from what it changed to.
text = +"hello world"
changing = Object.new
changing.define_singleton_method :to_int do
  text.replace "HELLO"
  1
end
p text.byteslice changing, 3
p text.byteslice 10
