# What the interpreter was built as and where it keeps its parts. Metorex is
# one binary rather than an installed tree, so the directories that only mean
# something after an installation are left out and `TOPDIR` is nil.

module RbConfig
  major, minor, teeny = RUBY_VERSION.split "."
  # The name the binary was started through, so one linked in as `ruby`
  # starts others as `ruby`.
  install_name = File.basename(__interpreter_path__() || "metorex")
  cpu, os = RUBY_PLATFORM.split "-", 2
  # The host as MRI's configure names it, which a library a gem builds for
  # it, such as nokogiri's libxml2, is kept under: arm64-apple-darwin27 on
  # macOS, where the operating system carries the kernel's major version.
  vendor = nil
  host = RUBY_PLATFORM.dup
  if os.include?("darwin")
    vendor = "apple"
    os = "#{os}#{__kernel_release__().to_i}"
    host = "#{cpu}-#{vendor}-#{os}"
  end

  CONFIG = {
    "MAJOR" => major,
    "MINOR" => minor,
    "TEENY" => teeny,
    "PATCHLEVEL" => RUBY_PATCHLEVEL.to_s,
    "ruby_version" => "#{major}.#{minor}.0",
    "RUBY_PROGRAM_VERSION" => RUBY_VERSION.dup,
    "RUBY_INSTALL_NAME" => install_name,
    "RUBY_BASE_NAME" => "metorex",
    "ruby_install_name" => install_name,
    "arch" => RUBY_PLATFORM.dup,
    "host" => host.dup,
    "host_alias" => "",
    "host_cpu" => cpu,
    "host_os" => os,
    "target" => host.dup,
    "target_alias" => "",
    "target_cpu" => cpu,
    "target_os" => os,
    "build" => host.dup,
    "build_alias" => "",
    "build_cpu" => cpu,
    "build_os" => os,
    "EXEEXT" => "",
    "CROSS_COMPILING" => "no",
    "DLEXT" => os.include?("darwin") ? "bundle" : "so",
    # The extensions metorex carries are built into the binary, and none is
    # loaded from a file of its own.
    "EXTSTATIC" => "static",
    "LIBEXT" => "a",
    "OBJEXT" => "o",
    "ENABLE_SHARED" => "no",
    "LIBRUBY" => "libmetorex-static.a",
    "LIBRUBY_SO" => "libmetorex.#{os.include?("darwin") ? "dylib" : "so"}",
    "LIBRUBY_A" => "libmetorex-static.a",
    "libdirname" => "libdir",
    "LIBPATHENV" => os.include?("darwin") ? "DYLD_FALLBACK_LIBRARY_PATH" : "LD_LIBRARY_PATH",
    "UNICODE_VERSION" => "17.0.0",
    "UNICODE_EMOJI_VERSION" => "17.0",
    "CC" => "cc",
    "CXX" => "c++",
    "AR" => "ar",
    "STRIP" => "strip",
    "SHELL" => "/bin/sh",
    "EXTOUT" => ".ext",
    "NULLCMD" => ":",
    "optflags" => "-O3 -fno-fast-math",
    "debugflags" => "-ggdb3",
    # Flags GCC and Clang both accept.
    "warnflags" => "-Wall -Wextra -Wdeprecated-declarations -Wimplicit-function-declaration " \
                   "-Wimplicit-int -Wpointer-arith -Wwrite-strings -Wold-style-definition " \
                   "-Wmissing-noreturn -Wno-long-long -Wno-missing-field-initializers " \
                   "-Wno-overlength-strings -Wno-unused-parameter -Wno-unused-value " \
                   "-Wunused-variable -Wundef",
    "WERRORFLAG" => "-Werror",
    "GCC" => "yes",
    "PKG_CONFIG" => "pkg-config",
    # An extension leaves the `rb_*` functions it calls for the binary that
    # loads it to supply.
    "DLDFLAGS" => os.include?("darwin") ? "-Wl,-undefined,dynamic_lookup" : "",
    "EXTDLDFLAGS" => ""
  }

  # A library installed alongside metorex goes under the prefix the binary
  # sits in, which is where `$LOAD_PATH` looks for one.
  prefix = File.dirname(File.dirname(__interpreter_path__()))
  # The directory the binary sits in, which a program that starts the
  # interpreter again joins with `ruby_install_name`.
  if vendor
    CONFIG["host_vendor"] = CONFIG["target_vendor"] = CONFIG["build_vendor"] = vendor
  end
  CONFIG["bindir"] = File.dirname(__interpreter_path__())
  CONFIG["prefix"] = prefix
  CONFIG["rubylibprefix"] = "#{prefix}/lib/metorex"
  CONFIG["rubylibdir"] = "#{CONFIG["rubylibprefix"]}/#{CONFIG["ruby_version"]}"
  CONFIG["sitelibdir"] = "#{prefix}/lib/metorex/#{RUBY_VERSION}"
  CONFIG["sitearchdir"] = "#{CONFIG["sitelibdir"]}/#{RUBY_PLATFORM}"
  CONFIG["rubyhdrdir"] = __header_directory__()
  CONFIG["rubyarchhdrdir"] = CONFIG["rubyhdrdir"]

  # Metorex runs from wherever its binary sits rather than from an installed
  # tree, so there is no prefix to name.
  TOPDIR = nil

  # The values a Makefile is written with, which an extconf.rb may change
  # without changing CONFIG.
  MAKEFILE_CONFIG = CONFIG.transform_values(&:dup)

  def self.expand(value, config = CONFIG)
    value
  end

  # The binary running this program, which starts another one.
  def self.ruby
    __interpreter_path__()
  end
end

# Whether Ruby was built for another platform than the one building it.
CROSS_COMPILING = nil unless defined? CROSS_COMPILING
