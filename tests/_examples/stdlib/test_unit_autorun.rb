# A file that requires test/unit and defines a test case runs its tests as
# the program ends, as `ruby file.rb` does with test-unit.
require "test/unit"

class InvoiceTotalsTest < Test::Unit::TestCase
  def test_adds_the_lines
    assert_equal(1320, [1200, 45, 75].sum)
  end
end
