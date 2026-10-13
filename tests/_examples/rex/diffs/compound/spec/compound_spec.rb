RSpec.describe "compound matchers" do
  let(:report) { "status: ok\nretries: 3\nregion: us-east-1\n" }

  it "diffs each failing matcher of an and" do
    expect(report).to eq("status: ok\nretries: 2\nregion: us-east-1\n").and eq("status: failed\nretries: 3\nregion: us-east-1\n")
  end

  it "diffs only the failing matcher" do
    expect(report).to start_with("status").and eq("status: ok\nretries: 4\nregion: us-east-1\n")
  end

  it "diffs both sides of an or" do
    expect(%w[alpha beta gamma]).to eq(%w[alpha beta]).or eq(%w[beta gamma])
  end

  it "diffs nested compounds" do
    expect({ id: 1, name: "web" }).to eq({ id: 2, name: "web" }).and(include(:id).and(eq({ id: 1, name: "db" })))
  end

  it "runs a block once for two block matchers" do
    counter = 0
    expect { counter += 1 }.to change { counter }.by(1).and output("").to_stdout
  end

  it "fails a block compound" do
    counter = 0
    expect { counter += 2 }.to change { counter }.by(1).and change { counter }.to(3)
  end

  it "refuses not_to with a compound" do
    expect(1).not_to eq(2).and eq(3)
  end

  it "refuses a block matcher with a value" do
    expect(1).to eq(1).and raise_error(ArgumentError)
  end
end
