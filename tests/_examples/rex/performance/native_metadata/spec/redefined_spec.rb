Rex::Core::Metadata::HashPopulator.class_exec do
  alias_method :rex_build_description_from, :build_description_from

  def build_description_from(parent_description = nil, my_description = nil)
    rex_build_description_from(parent_description, my_description).upcase
  end
end

RSpec.describe "an invoice" do
  it "is described by the redefined step" do |example|
    expect(example.metadata[:description]).to eq("IS DESCRIBED BY THE REDEFINED STEP")
  end
end
