# A library already loaded from where Ruby carries it is not loaded again
# from a file of the same name put on the load path afterwards.
require "power_assert"
$LOAD_PATH.unshift File.join(__dir__, "carried_then_on_load_path", "lib")
p require("power_assert")
p $LOADED_FEATURES.grep(/power_assert\.rb\z/).size
