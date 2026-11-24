["त्र", "क्षि", "न्दी", "क्‍ष", "a्b", "क्"].each do |text|
  p [text.grapheme_clusters.size, text.grapheme_clusters.map(&:size)]
end
p "क्‍ष".grapheme_clusters.size
