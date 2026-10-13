RSpec.configure { |config| config.order = :defined }

$resets = 0

Rex::Mocks::Space.class_exec do
  alias_method :rex_reset_all, :reset_all

  def reset_all
    $resets += 1
    rex_reset_all
  end
end

RSpec.describe "an example with no doubles after reset_all was redefined" do
  it("sums") { expect(1 + 1).to eq(2) }
  it("took its space down through the redefined method") { expect($resets).to eq(1) }
end
