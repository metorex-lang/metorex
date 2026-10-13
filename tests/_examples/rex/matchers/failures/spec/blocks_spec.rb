describe "block matchers" do
  it("raise_error with a class") { expect { 1 / 0 }.to raise_error(ArgumentError) }
  it("raise_error nothing raised") { expect { 1 }.to raise_error(ZeroDivisionError) }
  it("raise_error message") { expect { raise "boom" }.to raise_error("bang") }
  it("raise_error regexp") { expect { raise ArgumentError, "bad value" }.to raise_error(ArgumentError, /missing/) }
  it("raise_error negated") { expect { raise "boom" }.not_to raise_error }
  it("throw_symbol") { expect { throw :done }.to throw_symbol(:halt) }
  it("throw_symbol nothing") { expect { 1 }.to throw_symbol }
  it("change") do
    list = []
    expect { list << 1 }.to change { list.size }.by(2)
  end
  it("change from to") do
    count = 1
    expect { count += 1 }.to change { count }.from(0).to(2)
  end
  it("change not") do
    list = []
    expect { list << 1 }.not_to change { list.size }
  end
  it("change by_at_least") do
    total = 0
    expect { total += 1 }.to change { total }.by_at_least(2)
  end
  it("output") { expect { print "hello" }.to output("goodbye").to_stdout }
  it("output to stderr") { expect { 1 }.to output.to_stderr }
  it("yield_control") { expect { |probe| [].each(&probe) }.to yield_control }
  it("yield_control count") { expect { |probe| [1, 2].each(&probe) }.to yield_control.once }
  it("yield_with_args") { expect { |probe| [1].each(&probe) }.to yield_with_args(2) }
  it("yield_with_no_args") { expect { |probe| [1].each(&probe) }.to yield_with_no_args }
  it("yield_successive_args") { expect { |probe| [1, 2].each(&probe) }.to yield_successive_args(1, 3) }
end
