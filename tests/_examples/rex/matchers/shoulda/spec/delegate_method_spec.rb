require "forwardable"
require "shoulda-matchers"

Shoulda::Matchers.configure do |config|
  config.integrate do |with|
    with.test_framework :rspec
  end
end

class PostOffice
  def deliver = "delivered"
  def ship = "shipped"
  def deliver_package(options) = options
  def name = "Main Street"
  def title = "Downtown branch"
end

class ForwardingCourier
  extend Forwardable

  attr_reader :post_office

  def_delegators :post_office, :deliver

  def initialize
    @post_office = PostOffice.new
  end
end

class Courier
  attr_reader :post_office

  delegate :name, to: :post_office, prefix: true
  delegate :title, to: :post_office, prefix: :branch

  def initialize
    @post_office = PostOffice.new
  end

  def deliver
    post_office.ship
  end

  def deliver_package(options)
    post_office.deliver_package(options)
  end

  def ship
    "kept"
  end
end

class Account
  attr_reader :post_office

  delegate :name, to: :post_office, allow_nil: true
  delegate :title, to: :post_office, private: true
end

describe ForwardingCourier do
  it { should delegate_method(:deliver).to(:post_office) }
  it { should_not delegate_method(:ship).to(:post_office) }
end

describe Courier do
  it { should delegate_method(:deliver).to(:post_office).as(:ship) }
  it { should delegate_method(:name).to(:post_office).with_prefix }
  it { should delegate_method(:title).to(:post_office).with_prefix(:branch) }
  it { should delegate_method(:deliver_package).to(:post_office).with_arguments(expedited: true) }

  context "where an expectation does not hold" do
    it { should delegate_method(:ship).to(:post_office) }
    it { should_not delegate_method(:deliver).to(:post_office).as(:ship) }
  end
end

describe Account do
  it { should delegate_method(:name).to(:post_office).allow_nil }
  it { should delegate_method(:title).to(:post_office).with_private }

  context "where an expectation does not hold" do
    it { should delegate_method(:title).to(:post_office) }
  end
end
