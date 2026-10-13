require_relative "../lib/inventory"

describe "an inventory" do
  it "counts the items" do
    expect(Inventory.count).to eq(3)
  end
end

describe "a backtrace formatter" do
  let(:formatter) { RSpec::Core::BacktraceFormatter.new }
  let(:project_line) { "#{Dir.pwd}/lib/inventory.rb:5:in 'Inventory.count'" }
  let(:ruby_line) { "/usr/lib/ruby/3.4.0/set.rb:10:in 'Set#add'" }

  it "writes a project line relative to the working directory" do
    expect(formatter.backtrace_line(project_line)).to eq("./lib/inventory.rb:5:in 'Inventory.count'")
  end

  it "leaves out a line from Ruby's library" do
    expect(formatter.backtrace_line(ruby_line)).to be_nil
  end

  it "shows the whole backtrace when every line is filtered out" do
    expect(formatter.format_backtrace([ruby_line])).to eq([
      ruby_line,
      "",
      "  Showing full backtrace because every line was filtered out.",
      "  See docs for RSpec::Configuration#backtrace_exclusion_patterns and",
      "  RSpec::Configuration#backtrace_inclusion_patterns for more information."
    ])
  end

  it "shows every line once full backtrace is on" do
    formatter.full_backtrace = true
    expect(formatter.format_backtrace([project_line, ruby_line])).to eq(["./lib/inventory.rb:5:in 'Inventory.count'", ruby_line])
  end

  it "shows every line when no exclusion pattern is left" do
    formatter.exclusion_patterns = []
    expect(formatter.full_backtrace?).to be(true)
  end
end
