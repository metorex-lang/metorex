# A file on the load path that a library Ruby carries requires names itself
# as the file its methods were written in. test-unit's backtrace filter
# requires power_assert, which the load path holds here.
$LOAD_PATH.unshift File.join(__dir__, "disk_file_required_by_a_carried_library")
require "test/unit/util/backtracefilter"
file, line = PowerAssert.method(:start).source_location
p File.basename(file), line
