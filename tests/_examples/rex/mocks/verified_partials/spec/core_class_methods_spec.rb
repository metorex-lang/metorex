RSpec.describe "partial doubles of core class methods" do
  it "stubs a Kernel function" do
    expect(::Kernel).to receive(:warn).with("careful")
    ::Kernel.warn("careful")
  end

  it "stubs a native class method" do
    allow(File).to receive(:exist?).and_return(true)
    expect(File.exist?("/no/such/path")).to be true
  end

  it "checks a native class method's arguments" do
    allow(File).to receive(:exist?)
    File.exist?
  end

  it "restores the native class method" do
    expect(File.exist?("/no/such/path")).to be false
  end

  it "refuses a class method the class does not have" do
    allow(File).to receive(:no_such_method)
  end
end
