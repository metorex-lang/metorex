class Invoice
  def total
    "the method"
  end

  def label(total)
    total
  end

  def describe(amount, *items, currency: "USD", **options, &formatter)
    text = "#{amount} #{currency} for #{items.join(", ")} #{options.keys.inspect}"
    formatter ? formatter.call(text) : text
  end

  def rebind(amount)
    binding.local_variable_set(:amount, amount * 2)
    amount
  end

  def evaluated(amount)
    eval("amount + 1")
  end

  def held(callable)
    callable
  end

  def twice(count)
    [count, count]
  end
  alias_method :doubled, :twice
  define_method(:paired, instance_method(:twice))
end

invoice = Invoice.new
p invoice.label(42)
p invoice.label(nil)
p invoice.describe(10, "paper", "ink", currency: "EUR", rush: true)
p invoice.describe(5, "toner") { |text| text.upcase }
p invoice.rebind(4)
p invoice.evaluated(6)
p invoice.held(invoice.method(:total)).call
p invoice.held(-> { :called }).call
p [invoice.twice(1), invoice.doubled(2), invoice.paired(3)]
