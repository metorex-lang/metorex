later do
  class Reading
    def place = binding
  end
  p File.basename(Reading.new.place.source_location[0])
  p File.basename(binding.source_location[0])
end
later do
  p File.basename(eval("__FILE__"))
end
