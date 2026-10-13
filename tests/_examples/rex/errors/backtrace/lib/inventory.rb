require_relative "../vendor/gems/fake_client-1.2/lib/fake_client"

module Inventory
  def self.count
    FakeClient.fetch("/items")
  end
end
