class Printer
  def print_page(number) = "page #{number}"
end

describe "any instance expectations" do
  it "reports an instance that never received the message" do
    expect_any_instance_of(Printer).to receive(:print_page)
  end

  it "passes when one instance receives it" do
    expect_any_instance_of(Printer).to receive(:print_page).and_return("stubbed")
    expect(Printer.new.print_page(1)).to eq("stubbed")
  end
end
