class Mock
  def method_missing(level, message = nil)
    puts "missing #{level}: #{message}"
  end

  def respond_to_missing?(*) = true
end

mock = Mock.new
mock&.warn "safe"
text = "bare"
mock&.warn text

class Account
  def balance_of(other) = other&.balance
  def own = self&.secret
  def each_twice(&block) = [1, 2]&.each(&block)

  protected

  def balance = 10

  private

  def secret = :hidden
end

account = Account.new
p account.balance_of(Account.new)
p account.own
account.each_twice { |n| p n }
begin
  account&.secret
rescue NoMethodError => error
  puts error.message
end
held = nil
p held&.secret
def size_of(list) = list&.size
p [size_of([1]), size_of([1, 2]), size_of(nil), size_of("abc")]
