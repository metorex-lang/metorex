# RbConfig says Ruby was built for the platform it runs on, which Bundler
# reads through CROSS_COMPILING.
require "rbconfig"

p(CROSS_COMPILING)
p(RbConfig::CONFIG["CROSS_COMPILING"])
p(CROSS_COMPILING || RUBY_PLATFORM == RUBY_PLATFORM)
