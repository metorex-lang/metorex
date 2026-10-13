RSpec.describe "a group that defines initialize" do
  def initialize(*)
    super
    @opened_with = :initialize
  end

  let(:total) { 42 }

  it "runs its own initialize for each example" do
    expect([@opened_with, total]).to eq([:initialize, 42])
  end
end
