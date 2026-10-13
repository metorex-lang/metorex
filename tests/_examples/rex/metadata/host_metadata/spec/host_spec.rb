RSpec.configure do |config|
  config.shared_context_metadata_behavior = :apply_to_host_groups
end

RSpec.shared_context "with a database", :database do
  let(:connection) { "a connection" }
end

describe "a host group" do
  include_context "with a database"

  it "has the context's metadata" do |example|
    expect(example.metadata[:database]).to eq(true)
  end

  it "has the context's let" do
    expect(connection).to eq("a connection")
  end
end

describe "a group with the tag", :database do
  it "does not include the context" do
    expect { connection }.to raise_error(NameError)
  end
end
