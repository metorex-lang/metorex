RSpec.describe "a settings writer" do
  it "turns on the dark theme" do
    $theme = :dark
  end
end

RSpec.describe "a settings reader" do
  it "sees the theme the writer set" do
    expect($theme).to eq(:dark)
  end
end
