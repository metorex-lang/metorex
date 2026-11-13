# A required file that requires another lands in $LOADED_FEATURES after it,
# once its own body has run.
$LOAD_PATH.unshift __dir__
require "outer"
p $LOADED_FEATURES.last(2).map { |feature| File.basename(feature) }
p OUTER_SAW_ITSELF
