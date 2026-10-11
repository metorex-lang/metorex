module Rates
  def load_rates = :first_version
  extend(self)
end

module Rates
  def load_rates = :second_version
end

module Tariffs
  def lookup = :first_version
end

module Registry
  extend(Tariffs)
end

module Tariffs
  def lookup = :second_version
end

p(Rates.load_rates)
p(Registry.lookup)
