# What mkmf finds of a type's signedness, as fiddle's extconf.rb asks of
# size_t, and the configuration header it reads beside the C API's own.
require "mkmf"
require "tmpdir"

Dir.mktmpdir do |directory|
  Dir.chdir(directory) do
    p File.file?(File.join($arch_hdrdir, "ruby/config.h"))
    p File.read(File.join($arch_hdrdir, "ruby/config.h")).match?(/^#define\s+SIZEOF_SIZE_T\s/)
    p check_signedness("size_t", "stddef.h")
    p check_signedness("ptrdiff_t", "stddef.h")
    p check_signedness("no_such_type_t", "stddef.h")
    p $defs.grep(/SIGNEDNESS/)
  end
end
