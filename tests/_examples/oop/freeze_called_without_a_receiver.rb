class Settings
  def initialize(host)
    @host = host
  end

  def lock
    freeze
    frozen?
  end

  def lock_through_self
    self.freeze
    frozen?
  end

  def answer_of_freeze
    freeze
  end
end

p Settings.new("mail.example.com").lock
p Settings.new("mail.example.com").lock_through_self
held = Settings.new("mail.example.com")
p held.answer_of_freeze.equal?(held)
begin
  held.instance_variable_set(:@host, "other.example.com")
rescue FrozenError => error
  p error.class
end
