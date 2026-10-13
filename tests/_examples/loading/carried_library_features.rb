p require("nkf")
p require("nkf")
require "json"
p $LOADED_FEATURES.grep(/nkf|json\.rb/).map { |feature| File.basename(feature, ".*") }
p $LOADED_FEATURES.any? { |feature| feature.end_with?("/json.rb") }
