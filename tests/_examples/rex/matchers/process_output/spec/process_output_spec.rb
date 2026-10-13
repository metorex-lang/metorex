describe "output from any process" do
  it "captures a child process writing to stdout" do
    expect { system("echo from a child") }.to output("from a child\n").to_stdout_from_any_process
  end

  it "captures a child process writing to stderr" do
    expect { system("echo warning 1>&2") }.to output(/warning/).to_stderr_from_any_process
  end

  it "fails when the child writes something else" do
    expect { system("echo actual") }.to output("expected\n").to_stdout_from_any_process
  end

  it "still prints to stdout afterwards" do
    $stdout.puts "after capture"
    expect(1).to eq(1)
  end
end

describe "aggregate failures as metadata", :aggregate_failures do
  it "reports every failure in the example" do
    expect(1).to eq(2)
    expect(3).to eq(4)
  end

  it "passes when nothing fails" do
    expect(1).to eq(1)
  end
end
