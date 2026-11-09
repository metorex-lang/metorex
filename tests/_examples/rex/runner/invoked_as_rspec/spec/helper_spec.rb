puts "helper_spec loaded"
describe "a run started as RSpec's" do
  it "leaves spec_helper.rb to the specs that require it" do
    expect($LOADED_FEATURES.grep(/spec_helper/)).to be_empty
  end
end
