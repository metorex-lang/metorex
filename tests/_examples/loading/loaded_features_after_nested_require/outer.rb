require "inner"
OUTER_SAW_ITSELF = $LOADED_FEATURES.any? { |feature| feature.end_with?("/outer.rb") }
