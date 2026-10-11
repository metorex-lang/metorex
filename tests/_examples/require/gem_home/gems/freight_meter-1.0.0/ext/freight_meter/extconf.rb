require "mkmf"

$INCFLAGS << " -I$(srcdir)/../../include"
root = File.expand_path("../..", __dir__)
$srcs = Dir.glob("#{root}/src/*.c") + Dir.glob("#{__dir__}/*.c")
$distcleanfiles << "meter_version.h"
File.write("meter_version.h", "#define FREIGHT_METER_VERSION \"1.0.0\"\n")
append_cflags ["-std=gnu99"]
create_makefile "freight_meter/meter"
