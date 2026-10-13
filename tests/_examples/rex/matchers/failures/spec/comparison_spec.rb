describe "comparison matchers" do
  it("be >") { expect(3).to be > 4 }
  it("be <=") { expect(5).to be <= 4 }
  it("be > negated") { expect(5).not_to be > 4 }
  it("be ==") { expect(5).to be == 4 }
  it("be_between") { expect(10).to be_between(1, 5) }
  it("be_between exclusive") { expect(5).to be_between(1, 5).exclusive }
  it("be_within") { expect(3.3).to be_within(0.1).of(3.0) }
  it("be_within percent") { expect(120).to be_within(10).percent_of(100) }
  it("be_a") { expect("text").to be_a(Integer) }
  it("be_an_instance_of") { expect(1).to be_an_instance_of(Numeric) }
  it("be_kind_of negated") { expect(1).not_to be_kind_of(Integer) }
  it("respond_to") { expect("text").to respond_to(:shout) }
  it("respond_to with arity") { expect("text").to respond_to(:upcase).with(2).arguments }
end
