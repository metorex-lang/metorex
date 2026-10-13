RSpec.configure { |config| config.raise_on_warning = true }

RSpec.describe "a warning" do
  it "is raised as an error" do
    expect { raise ArgumentError }.to raise_error
  end

  it "is not raised for a specific error class" do
    expect { raise ArgumentError }.to raise_error(ArgumentError)
  end
end
