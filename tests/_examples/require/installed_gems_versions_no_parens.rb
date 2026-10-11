# `gem` activates the version a requirement asks for, and refuses another
# version once one is active, a version nothing installed has, and a gem
# that is not installed at all. The count of gems Ruby reports takes in the
# ones it ships with, so it is left out.
def report
  yield
rescue LoadError => error
  p([error.class, error.message.lines.first.chomp.sub(/among \d+/, "among N")])
end

p gem("shipping_rates", "< 1.2")
p gem("shipping_rates", "1.0.0")
require "shipping_rates"
p ShippingRates::VERSION
p Gem::Specification.find_by_name("shipping_rates").full_name
report { gem "shipping_rates", "1.2.0" }
report { gem "tariff_table", "> 9" }
report { gem "parcel_labels" }
p Gem::Version.new("1.2.0") > Gem::Version.new("1.10.0")
p Gem::Requirement.new("~> 1.2").satisfied_by?(Gem::Version.new("1.9"))
