RSpec.describe "A UTF-16LE string" do
  let(:space) { " ".encode("UTF-16LE") }
  let(:word) { "carté".encode("UTF-16LE") }
  let(:newline) { "\n".encode("UTF-16LE") }

  it "keeps its encoding through interpolation" do
    expect("#{space}#{word}".encoding).to eq(Encoding::UTF_16LE)
  end

  it "keeps its characters through interpolation" do
    expect("#{space}#{word}".encode("UTF-8")).to eq(" carté")
  end

  it "replaces a character by index" do
    built = String.new("#{space}#{word}")
    built[0, 1] = "-".encode("UTF-16LE")
    expect(built.encode("UTF-8")).to eq("-carté")
  end

  it "slices by character" do
    expect([word[0, 1], word[1..2], word[-1]].map { |part| part.encode("UTF-8") }).to eq(["c", "ar", "é"])
  end

  it "joins with a separator in its own encoding" do
    joined = [word, word].join(newline)
    expect([joined.encoding, joined.encode("UTF-8")]).to eq([Encoding::UTF_16LE, "carté\ncarté"])
  end
end
