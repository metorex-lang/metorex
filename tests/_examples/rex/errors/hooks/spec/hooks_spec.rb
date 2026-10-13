describe "a before(:context) hook that raises" do
  before(:context) { raise "the database is down" }

  it "fails the first example" do
  end

  it "fails the second example" do
  end
end

describe "an after hook that raises" do
  after { raise ArgumentError, "the cleanup failed" }

  it "passes on its own" do
    expect(1).to eq(1)
  end

  it "fails on its own too" do
    expect(1).to eq(2)
  end
end

describe "an after(:context) hook that raises" do
  after(:context) { raise "the teardown failed" }

  it "passes" do
  end
end

describe "an around hook that raises" do
  around do |example|
    example.run
    raise "the wrapper failed"
  end

  it "passes before the wrapper raises" do
  end
end

describe "a before hook that raises" do
  before { raise IOError, "the fixture file is missing" }
  after { raise "the cleanup failed as well" }

  it "reports both errors" do
  end
end
