require "rspec/matchers/fail_matchers"

RSpec.describe "Matchers a group defines" do
  include RSpec::Matchers::FailMatchers

  matcher :divide_by_two do
    match { |actual| actual.even? }
  end
  alias_matcher :a_number_dividing_by_two, :divide_by_two
  define_negated_matcher :leave_a_remainder, :divide_by_two

  it "matches with a matcher the group defines" do
    expect(2).to divide_by_two
  end

  it "describes an alias by its own name" do
    expect(a_number_dividing_by_two.description).to eq("a number dividing by two")
  end

  it "negates a matcher the group defines" do
    expect(3).to leave_a_remainder
  end

  it "leaves Rex::Matchers without the group's matchers" do
    expect(RSpec::Matchers.method_defined?(:leave_a_remainder)).to be false
  end

  it "fails an alias with its own name in the message" do
    expect([3]).to include(a_number_dividing_by_two)
  end

  it "matches a failing expectation with fail_with" do
    expect { expect(3).to divide_by_two }.to fail_with("expected 3 to divide by two")
  end

  it "matches a failing expectation with fail_including" do
    expect { expect(3).to divide_by_two }.to fail_including("to divide by two")
  end

  it "matches any failing expectation with fail" do
    expect { expect(3).to divide_by_two }.to fail
  end
end

RSpec.describe "A sibling group" do
  it "does not answer the other group's matchers" do
    expect(respond_to?(:divide_by_two)).to be false
  end
end
