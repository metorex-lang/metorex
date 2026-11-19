# Whether a source preprocesses, as nokogiri's extconf.rb asks of the
# libxml2 version a header declares.
require "mkmf"
require "tmpdir"

Dir.mktmpdir do |directory|
  Dir.chdir(directory) do
    p try_cpp("#include <stddef.h>\n#if 1\nint preprocessed;\n#endif\n")
    p try_cpp("#if 0 < 1\n#error the check fails\n#endif\n")
    p try_cpp("#include <no_such_header_here.h>\n")
  end
end
