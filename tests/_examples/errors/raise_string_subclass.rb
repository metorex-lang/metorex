# `raise` with an instance of a String subclass raises a RuntimeError with
# that string as its message, as Rainbow's presenters are raised.
class Presenter < String
  def red
    self
  end
end

begin
  raise Presenter.new("Formatter not found.").red
rescue => error
  p error.class
  p error.message
  p error.message.class
end
