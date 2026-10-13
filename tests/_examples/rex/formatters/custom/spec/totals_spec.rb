require "spec_helper"

describe "an account" do
  it "opens" do
    expect(:open).to eq(:open)
  end

  it "closes" do
    expect(:open).to eq(:closed)
  end

  it "freezes", skip: "the freeze rule" do
  end
end
