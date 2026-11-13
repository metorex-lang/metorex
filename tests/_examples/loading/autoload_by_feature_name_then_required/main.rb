# An autoload registered by feature name, whose file is then required
# directly, is no autoload any more once the file has defined the constant,
# also after the constant was removed and the feature list put back, as
# Tilt's mapping tests do between examples.
$LOAD_PATH << __dir__
2.times do
  loaded_before = $LOADED_FEATURES.dup
  Object.autoload :FeatureTemplate, "feature_template"
  p Object.autoload?(:FeatureTemplate)
  p require("feature_template")
  p [Object.autoload?(:FeatureTemplate), Object.autoload?("FeatureTemplate"), Object.const_defined?("FeatureTemplate")]
  p FeatureTemplate.name
  Object.send(:remove_const, :FeatureTemplate)
  $LOADED_FEATURES.replace(loaded_before)
end
