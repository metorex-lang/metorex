describe Greetings do
  it "greets by name" do
    expect(Greetings.for("Ada")).to eq("Hello, Ada")
  end
end
