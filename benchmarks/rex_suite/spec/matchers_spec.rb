RSpec.describe "built-in matchers" do
  100.times do |index|
    it("includes #{index}") { expect([index, index + 1]).to include(index) }
    it("matches #{index}") { expect("order #{index}").to match(/order \d+/) }
    it("is nil for #{index}") { expect(nil).to be_nil }
    it("is truthy for #{index}") { expect(index).to be_truthy }
    it("is the same object for #{index}") { expect(:held).to be(:held) }
  end
end
