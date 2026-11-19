# What mkmf finds of the C API functions ruby.h does not declare, as json's
# parser asks of rb_enc_raise, and the Makefile it writes for an extconf.rb
# next to no sources, as json's top one is: one that builds and installs no
# library.
require "mkmf"
require "tmpdir"

Dir.mktmpdir do |directory|
  Dir.chdir(directory) do
    $srcdir = directory

    p have_func("rb_enc_raise", "ruby.h")
    p have_func("rb_enc_vsprintf", "ruby.h")

    create_makefile("nothing")
    built = system("make", "-s", out: File::NULL, err: File::NULL)
    installed = File.join(directory, "installed")
    install = system("make", "-s", "install", "sitearchdir=#{installed}", "sitelibdir=#{installed}", out: File::NULL, err: File::NULL)
    p [built, install]
    p Dir.glob("*.#{RbConfig::CONFIG["DLEXT"]}")
    p Dir.exist?(installed) ? Dir.children(installed) : []
  end
end
