# A C extension using the helpers libraries such as nokogiri and sqlite3
# reach for: ruby/defines.h, ruby_strdup, st's case-insensitive comparisons,
# CONST_ID, ALLOCV, RB_NIL_P, RB_NUM2LONG, rb_pid_t, and rb_memerror.
require "tmpdir"
require_relative "build_helper"

directory = Dir.mktmpdir
require(build_extension("c_library_helpers.c", "c_library_helpers", directory))
helpers = CLibraryHelpers.new

p(helpers.copy("maintenance window"))
p(helpers.compare("Content-Type", "content-type"), helpers.compare("abc", "abd"), helpers.compare("b", "A"))
p(helpers.compare_prefix("Accept-Encoding", "ACCEPT-language", 7), helpers.compare_prefix("Accept", "Accent", 4))
p(helpers.named("2026-11"))
p(helpers.total([120, 300, 45]))
p(helpers.missing(nil), helpers.missing(false))
p(helpers.process)
begin
  helpers.out_of_memory
rescue NoMemoryError => error
  puts "#{error.class}: #{error.message}"
end
