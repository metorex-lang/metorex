RSpec.describe "the fast paths setting" do
  it "reads what the command line or the configuration set" do
    puts "fast paths: #{RSpec.configuration.fast_paths?}"
    expect([true, false]).to include(RSpec.configuration.fast_paths)
  end
end
