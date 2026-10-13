describe "a file that fails to load" do
  it "never registers" do
    expect(1).to eq(1)
  end
end

UndefinedHelper.configure
