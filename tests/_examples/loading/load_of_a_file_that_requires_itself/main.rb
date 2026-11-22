$runs = []
p load(File.expand_path("requires_itself.rb", __dir__))
p $runs
p $LOADED_FEATURES.count { |feature| feature.end_with?("requires_itself.rb") }
