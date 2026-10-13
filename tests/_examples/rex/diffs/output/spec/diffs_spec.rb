Point = Struct.new(:x, :y)

describe "diffs" do
  it "shows one hunk for a change in a multi-line String" do
    expect("alpha\nbeta\ngamma\ndelta\n").to eq("alpha\nbeta\nGAMMA\ndelta\n")
  end

  it "shows separate hunks for changes far apart" do
    actual = (1..20).map { |line| "line #{line}" }
    expected = actual.dup
    expected[1] = "line two"
    expected[17] = "line eighteen"
    expect(actual.join("\n")).to eq(expected.join("\n"))
  end

  it "merges hunks whose context overlaps" do
    actual = (1..12).map { |line| "row #{line}" }
    expected = actual.dup
    expected[2] = "row three"
    expected[8] = "row nine"
    expect(actual.join("\n")).to eq(expected.join("\n"))
  end

  it "pairs lines as diff-lcs does when two pairings are equally long" do
    expect("b\na\nc\n").to eq("a\nb\nc\n")
  end

  it "shows added and removed lines at the ends" do
    expect("middle\nend\nextra").to eq("start\nmiddle\nend")
  end

  it "diffs a Hash by its sorted keys" do
    expect({ name: "Ada", role: "admin", age: 36 }).to eq({ name: "Ada", role: "editor", age: 36, team: "core" })
  end

  it "diffs an Array" do
    expect([1, :two, "three", [4]]).to eq([1, :two, "three", [5]])
  end

  it "diffs a Struct through pretty_print" do
    expect(Point.new(1, 2)).to eq(Point.new(1, 3))
  end

  it "diffs Arrays of Strings as their lines" do
    expect(%w[apple banana cherry]).to eq(%w[apple blueberry cherry])
  end

  it "prints no diff for one-line Strings" do
    expect("short").to eq("shorter")
  end

  it "prints no diff for numbers" do
    expect(41).to eq(42)
  end

  it "diffs what include was given against the Hash" do
    expect({ a: 1, b: 2 }).to include(a: 1, c: 3)
  end

  it "diffs a multi-line String against a pattern" do
    expect("first\nsecond").to match(/first\nthird/)
  end
end
