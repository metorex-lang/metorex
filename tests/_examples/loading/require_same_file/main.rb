$LOAD_PATH.unshift(__dir__)
p require("helper_once")
p require(File.join(__dir__, "helper_once.rb"))
p require(File.join(__dir__, "helper_once"))
p LOADS
