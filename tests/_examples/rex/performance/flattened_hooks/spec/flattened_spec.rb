RSpec.describe "hooks around an example" do
  before { @order = [:outer_before] }
  after { $order_after << :outer_after }
  around do |example|
    $order_around = [:outer_around]
    example.run
  end

  context "nested a level down" do
    before { @order << :inner_before }
    after { $order_after = [:inner_after] }
    around do |example|
      $order_around << :inner_around
      example.run
    end

    it "runs the hooks in RSpec's order" do
      expect([@order, $order_around]).to eq([[:outer_before, :inner_before], [:outer_around, :inner_around]])
    end

    it "runs the group's hooks from one list built for it" do
      expect(Rex::Core::FastPaths::FLATTENED_HOOKS.key?(self.class)).to be(true)
    end
  end
end
