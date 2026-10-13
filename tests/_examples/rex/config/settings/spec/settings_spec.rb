class ConnectionError < StandardError; end

def connect
  raise IOError, "socket closed"
rescue IOError
  raise ConnectionError, "could not reach the ledger service"
end

RSpec.describe "failures with a cause" do
  it "prints the cause under the failure" do
    connect
  end
end

RSpec.describe "pending examples" do
  it "fails as expected" do
    pending "the rounding rule"
    expect(1.005.round(2)).to eq(1.0051)
  end

  it "is skipped", skip: "the export format" do
  end
end

RSpec.describe "metadata derived by the configuration" do
  it "is slow because it is in an integration file" do |example|
    expect(example.metadata[:slow]).to eq(SLOW_EXPECTED)
  end
end

RSpec.describe "hooks with a scope and metadata", :audited do
  it "ran the audited before(:context) hook" do
    expect(AUDIT_LOG).to eq([:context])
  end

  it "runs the audited before(:example) hook", :tracked do
    expect(AUDIT_LOG).to eq(%i[context example])
  end
end
