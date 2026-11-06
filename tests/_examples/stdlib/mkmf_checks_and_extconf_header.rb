# What an extconf.rb asks mkmf: whether the interpreter's functions link,
# whether the compiler takes a flag, the values the Makefile is written
# with, and the header that carries the checks' definitions.
require "mkmf"
require "tmpdir"

Dir.mktmpdir do |directory|
  Dir.chdir(directory) do
    $srcdir = directory

    p have_func("rb_hash_new_capa", "ruby.h")
    p have_func("rb_enc_interned_str", "ruby.h")
    p have_func("rb_no_function_has_this_name")

    append_cflags("-Wall")
    append_cflags("-Wno-flag-the-compiler-knows")
    p $CFLAGS.split.include?("-Wall")
    p $CFLAGS.include?("-Wno-flag-the-compiler-knows")

    p CONFIG.equal?(RbConfig::MAKEFILE_CONFIG)
    CONFIG["optflags"] << " -fno-strict-aliasing"
    p RbConfig::CONFIG["optflags"].include?("-fno-strict-aliasing")
    p config_string("WERRORFLAG")
    p config_string("no such key")
    p $DLDFLAGS.class

    $defs << "-DTHE_ANSWER=42"
    create_header
    puts File.read("extconf.h")

    create_makefile("checked")
    makefile = File.read("Makefile")
    puts makefile[/^RUBY_EXTCONF_H = .*/]
    p makefile.include?(%(-DRUBY_EXTCONF_H=\\"$(RUBY_EXTCONF_H)\\"))
    p makefile.include?("-DTHE_ANSWER")
    p makefile.include?("optflags = #{CONFIG["optflags"]}")
  end
end
