describe "equality matchers" do
  it("eq") { expect(4).to eq(5) }
  it("eq negated") { expect(5).not_to eq(5) }
  it("eq strings") { expect("checkout").to eq("check out") }
  it("eq arrays") { expect([1, 2, 3]).to eq([1, 2, 4]) }
  it("eq hashes") { expect({ name: "Ada", role: :admin }).to eq({ name: "Ada", role: :owner }) }
  it("eq multi-line strings") { expect("line one\nline two\nline three\n").to eq("line one\nline 2\nline three\n") }
  it("eql") { expect(1).to eql(1.0) }
  it("eql negated") { expect(1).not_to eql(1) }
  it("equal true") { expect(false).to equal(true) }
  it("equal nil") { expect(1).to be(nil) }
  it("be") { expect(nil).to be }
  it("be negated") { expect(1).not_to be }
  it("be_truthy") { expect(nil).to be_truthy }
  it("be_falsey") { expect(1).to be_falsey }
  it("be_nil") { expect(1).to be_nil }
  it("be_nil negated") { expect(nil).not_to be_nil }
end
