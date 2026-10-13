$matched_by = []

[
  RSpec::Matchers::BuiltIn::Eq, RSpec::Matchers::BuiltIn::Equal, RSpec::Matchers::BuiltIn::BeTruthy,
  RSpec::Matchers::BuiltIn::BeNil, RSpec::Matchers::BuiltIn::Match, RSpec::Matchers::BuiltIn::Include
].each do |matcher_class|
  matcher_class.class_exec do
    alias_method :rex_matches?, :matches?

    define_method(:matches?) do |actual, &block|
      $matched_by << matcher_class.name.split("::").last
      rex_matches?(actual, &block)
    end
  end
end

RSpec.describe "matchers whose matches? was redefined" do
  it "run each redefinition" do
    expect(42).to eq(42)
    expect(:ledger).to be(:ledger)
    expect("open").to be_truthy
    expect(nil).to be_nil
    expect("invoice 1001").to match(/\d+/)
    expect([12, 30]).to include(30)
    expect($matched_by.uniq).to eq(%w[Eq Equal BeTruthy BeNil Match Include])
  end
end
