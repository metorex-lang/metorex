# An extconf.rb runs commands through mkmf's xsystem and xpopen, as sqlite3's
# does to ask pkg-config for flags, and reads the host its libraries are
# built for from RbConfig, as nokogiri's does.
require "mkmf"
require "tmpdir"

Dir.chdir(Dir.mktmpdir)

p xsystem(["true"])
p xsystem([{ "LEDGER_MONTH" => "2026-11" }, "sh", "-c", 'test "$LEDGER_MONTH" = 2026-11'])
p xpopen(["echo", "maintenance window"], &:read)
p xpopen([{ "LEDGER_MONTH" => "2026-11" }, "sh", "-c", 'echo "$LEDGER_MONTH"'], err: [:child, :out], &:read)
p xpopen("echo shell words", &:read)

config = RbConfig::CONFIG
p config.values_at("host_alias", "build_alias", "target_alias")
p config["host"] == [config["host_cpu"], config["host_vendor"], config["host_os"]].compact.join("-")

# find_library takes a library the linker finds without a directory, and
# tries each directory named after it in turn.
p find_library("z", "zlibVersion")
p find_library("m", "cos", "/nonexistent", "/usr/lib")
p $libs.split

# The file name endings an extconf.rb builds the name of a static library or
# an object file from.
p [$LIBEXT, $OBJEXT, $EXEEXT]
p [$INSTALLFILES, $LOCAL_LIBS]
