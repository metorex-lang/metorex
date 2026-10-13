RSpec.configure do |config|
  config.register_ordering(:reverse) { |items| items.reverse }
  config.register_ordering(:global) { |items| items.sort_by(&:description) }
end

describe "zebra" do
  it "runs first by name" do
  end

  it "runs second by name" do
  end
end

describe "apple", order: :reverse do
  it "is defined first" do
  end

  it "is defined second" do
  end

  it "is defined third" do
  end
end

describe "mango", order: :unknown do
  it "falls back to the global ordering" do
  end
end
