describe "collection matchers" do
  it("include") { expect([1, 2, 3]).to include(4) }
  it("include several") { expect([1, 2, 3]).to include(4, 5) }
  it("include negated") { expect([1, 2, 3]).not_to include(2) }
  it("include string") { expect("checkout").to include("cart") }
  it("include hash pair") { expect({ a: 1, b: 2 }).to include(c: 3) }
  it("contain_exactly") { expect([1, 2, 3]).to contain_exactly(1, 2, 4) }
  it("match_array") { expect([3, 1]).to match_array([1, 2, 3]) }
  it("contain_exactly negated") { expect([1, 2]).not_to contain_exactly(2, 1) }
  it("start_with") { expect("checkout").to start_with("cart") }
  it("end_with array") { expect([1, 2, 3]).to end_with(2) }
  it("all") { expect([1, 2, 3]).to all(be_odd) }
  it("match regexp") { expect("order 42").to match(/invoice \d+/) }
  it("match nested") { expect({ id: 1, tags: ["a"] }).to match({ id: Integer, tags: [String, String] }) }
  it("cover") { expect(1..5).to cover(7) }
end
