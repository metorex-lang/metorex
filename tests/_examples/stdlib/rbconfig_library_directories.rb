require "rbconfig"

prefix = RbConfig::CONFIG["rubylibprefix"]
p prefix.start_with?(RbConfig::CONFIG["prefix"] + "/lib/")
p RbConfig::CONFIG["rubylibdir"] == File.join(prefix, RbConfig::CONFIG["ruby_version"])
p File.expand_path(prefix) == prefix
