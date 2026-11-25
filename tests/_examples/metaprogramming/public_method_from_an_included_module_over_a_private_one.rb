class Persisted
  private

  def load_session(request)
    "private #{request}"
  end
end

module StaleSessionCheck
  def load_session(request)
    "checked #{super}"
  end
end

store_class = Class.new(Persisted) do
  include StaleSessionCheck
end

p store_class.new.load_session("req")
p store_class.public_method_defined?(:load_session)
begin
  Persisted.new.load_session("req")
rescue NoMethodError => error
  puts error.message
end
