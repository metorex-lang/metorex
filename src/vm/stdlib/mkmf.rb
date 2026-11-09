# The helpers an extconf.rb calls to describe how to build a C extension,
# the checks it runs to learn what the system has, and the Makefile they
# write. The extension is compiled against the headers metorex ships, and the
# `rb_*` functions it calls are left for the metorex binary that loads it to
# supply.

require 'rbconfig'
require 'shellwords'
require 'tmpdir'

class String
  # The name a C preprocessor symbol is spelled with for this text.
  def tr_cpp
    strip.upcase.tr_s("^A-Z0-9_*", "_").tr_s("*", "P")
  end

  def funcall_style
    /\)\z/ =~ self ? dup : "#{self}()"
  end

  # The text in double quotes when it holds whitespace, as a Makefile
  # needs it.
  def quote
    /\s/ =~ self ? "\"#{self}\"" : "#{self}"
  end

  def sans_arguments
    self[/\A[^()]+/]
  end
end

class Array
  def quote
    map(&:quote)
  end
end

module MakeMakefile
  # The values the Makefile is written with.
  CONFIG = RbConfig::MAKEFILE_CONFIG

  # How a library, and a directory to find libraries in, are named to the
  # linker.
  LIBARG = "-l%s"
  LIBPATHFLAG = " -L%s"

  # The libraries every program links against already, which need no check.
  COMMON_LIBS = [].freeze

  # What every source a check compiles starts with.
  COMMON_HEADERS = "#include \"ruby.h\"\n"

  # Whether `message` keeps quiet, which `$VERBOSE` overrides.
  module Logging
    @quiet = $extmk

    class << self
      attr_accessor :quiet
    end
  end

  # A `main` that does nothing, so a source links into a program.
  MAIN_DOES_NOTHING = "int main(int argc, char **argv)\n{\n  return !!argv[argc];\n}\n"

  # Append each flag the compiler accepts without a warning to $CFLAGS.
  def append_cflags(flags, **options)
    Array(flags).each do |flag|
      if checking_for("whether #{flag} is accepted as CFLAGS") { try_cflags(flag, **options) }
        $CFLAGS << " " << flag
      end
    end
  end

  def try_cflags(flags, werror: true, **_options)
    try_compile(MAIN_DOES_NOTHING, flags, werror: werror)
  end

  # The value CONFIG holds for `key`, or what the block answers for it, when
  # the value is not empty.
  def config_string(key, config = CONFIG)
    value = config[key]
    return value if value.nil? || value.empty?

    block_given? ? yield(value) : value
  end

  # Append each flag the linker accepts to $LDFLAGS.
  def append_ldflags(flags, **_options)
    Array(flags).each do |flag|
      if checking_for("whether #{flag} is accepted as LDFLAGS") { try_ldflags(flag) }
        $LDFLAGS << " " << flag
      end
    end
  end

  def try_ldflags(flags, **_options)
    try_link(MAIN_DOES_NOTHING, "", ldflags: flags)
  end

  # The path of the program `bin` names, searched for on `path` or $PATH.
  def find_executable0(bin, path = nil)
    executable = ->(file) { File.file?(file) && File.executable?(file) }
    return (bin if executable.call(bin)) if File.expand_path(bin) == bin

    directories = (path || ENV["PATH"] || "/usr/local/bin:/usr/ucb:/usr/bin:/bin").split(File::PATH_SEPARATOR)
    directories.map { |directory| File.join(directory, bin) }.find(&executable)
  end

  def find_executable(bin, path = nil)
    checking_for checking_message(bin, path) do
      find_executable0(bin, path)
    end
  end

  # The compile and link flags pkg-config, or the package's own config
  # command, reports for `pkg`, added to $INCFLAGS, $CFLAGS, $LDFLAGS and
  # $libs, as [cflags, ldflags, libs]. Given options, the command's answer to
  # them is returned instead, and nothing is added.
  def pkg_config(pkg, *options)
    not_found = "not found"
    def not_found.%(answer)
      answer ? answer.inspect : self
    end

    checking_for "pkg-config for #{pkg}", not_found do
      _, library_directory = dir_config(pkg)
      environment = {}
      if library_directory && File.directory?(search_path = "#{library_directory}/pkgconfig")
        environment["PKG_CONFIG_PATH"] = [search_path, ENV["PKG_CONFIG_PATH"]].compact.join(File::PATH_SEPARATOR)
      end
      arguments = []
      if (command = with_config("#{pkg}-config")) && find_executable0(command)
        # The package's own config command, named on the command line.
      elsif ($PKGCONFIG ||= (command = with_config("pkg-config") { config_string("PKG_CONFIG") || ENV["PKG_CONFIG"] || "pkg-config" }) &&
                            find_executable0(command) && command) &&
            system(environment, $PKGCONFIG, "--exists", pkg)
        command = $PKGCONFIG
        arguments = [pkg]
      elsif find_executable0(command = "#{pkg}-config")
        # The package's own config command, found on the path.
      else
        command = nil
      end
      if command
        ask = proc do |asked|
          flags = Array(asked).map { |option| "--#{option}" }
          answer = IO.popen(environment, [command, *flags, *arguments], err: [:child, :out], &:read)
          if $?.success?
            answer.strip.shellsplit.map do |word|
              if word.start_with?("-l")
                LIBARG % word[2..]
              elsif word.start_with?("-L")
                (LIBPATHFLAG % word[2..]).strip
              else
                word
              end
            end.quote.join(" ")
          end
        end
      end
      if ask && !options.empty?
        ask[options]
      elsif ask && try_ldflags(ldflags = ask["libs"])
        if (incflags = ask["cflags-only-I"])
          $INCFLAGS << " " << incflags
          cflags = ask["cflags-only-other"]
        else
          cflags = ask["cflags"]
        end
        libs = ask["libs-only-l"]
        $CFLAGS += " " << cflags if cflags
        if libs
          ldflags = (Shellwords.shellwords(ldflags) - Shellwords.shellwords(libs)).quote.join(" ")
        else
          libs, ldflags = Shellwords.shellwords(ldflags).partition { |word| word =~ /-l([^ ]+)/ }.map { |words| words.quote.join(" ") }
        end
        $libs += " " << libs
        $LDFLAGS = [$LDFLAGS, ldflags].join(" ")
        [[incflags, cflags].join(" "), ldflags, libs]
      end
    end
  end

  # Write the definitions the checks made to `header`, which the Makefile
  # then has the sources include through RUBY_EXTCONF_H in place of passing
  # each one on the command line.
  def create_header(header = "extconf.h")
    message "creating %s\n", header
    guard = header.tr_cpp
    lines = ["#ifndef #{guard}\n#define #{guard}\n"]
    $defs.each do |definition|
      case definition
      when /^-D([^=]+)(?:=(.*))?/
        lines << "#define #{$1} #{$2 ? Shellwords.shellwords($2)[0].gsub(/(?=\t+)/, "\\\n") : 1}\n"
      when /^-U(.*)/
        lines << "#undef #{$1}\n"
      end
    end
    lines << "#endif\n"
    text = lines.join
    File.binwrite(header, text) unless (File.read(header) == text rescue false)
    $extconf_h = header
  end

  # The value `--name=value` gave on the command line, the default when it
  # was not given, or what the block answers then.
  def arg_config(config, default = nil, &block)
    $arg_config << [config, default]
    defaults = []
    if default
      defaults << default
    elsif !block
      defaults << nil
    end
    $configure_args.fetch(config.tr("_", "-"), *defaults, &block)
  end

  # What `--with-name` or `--without-name` said, with "yes" and "no" read as
  # true and false.
  def with_config(config, default = nil)
    config = config.sub(/^--with[-_]/, "")
    value = arg_config("--with-" + config) do
      if arg_config("--without-" + config)
        false
      elsif block_given?
        yield(config, default)
      else
        break default
      end
    end
    case value
    when "yes" then true
    when "no" then false
    else value
    end
  end

  # What `--enable-name` or `--disable-name` said, or the default when
  # neither was given, or what the block answers then.
  def enable_config(config, default = nil)
    if arg_config("--enable-" + config)
      true
    elsif arg_config("--disable-" + config)
      false
    elsif block_given?
      yield(config, default)
    else
      default
    end
  end

  # Run the block with $CFLAGS set to `flags`, keeping them when the block
  # answers true and putting the old ones back otherwise.
  def with_cflags(flags)
    held = $CFLAGS
    $CFLAGS = flags.dup
    answered = yield
  ensure
    $CFLAGS = held unless answered
  end

  def with_cppflags(flags)
    held = $CPPFLAGS
    $CPPFLAGS = flags.dup
    answered = yield
  ensure
    $CPPFLAGS = held unless answered
  end

  def with_ldflags(flags)
    held = $LDFLAGS
    $LDFLAGS = flags.dup
    answered = yield
  ensure
    $LDFLAGS = held unless answered
  end

  # The include and library directories `--with-target-dir`,
  # `--with-target-include` and `--with-target-lib` name, or the defaults
  # given, put ahead of the ones searched already.
  def dir_config(target, include_default = nil, lib_default = nil)
    key = [target, include_default, lib_default].compact.join("\0")
    if (configured = $config_dirs[key])
      return configured
    end

    if (dir = with_config(target + "-dir", (include_default unless lib_default)))
      defaults = dir.is_a?(Array) ? dir : dir.split(File::PATH_SEPARATOR)
      include_default = lib_default = nil
    end

    include_dir = with_config(target + "-include", include_default)
    if (given = $arg_config.assoc("--with-#{target}-include"))
      given[1] ||= "${#{target}-dir}/include"
    end
    lib_dir = with_config(target + "-lib", lib_default)
    if (given = $arg_config.assoc("--with-#{target}-lib"))
      given[1] ||= "${#{target}-dir}/lib"
    end

    include_dirs = include_dir ? (include_dir.is_a?(Array) ? include_dir.dup : include_dir.split(File::PATH_SEPARATOR)) : []
    if defaults
      include_dirs.concat(defaults.map { |held| held + "/include" })
      include_dir = ([include_dir] + include_dirs).compact.join(File::PATH_SEPARATOR)
    end
    unless include_dirs.empty?
      flags = include_dirs.map { |held| "-I" + held } - Shellwords.shellwords($CPPFLAGS)
      $CPPFLAGS = (flags.quote << $CPPFLAGS).join(" ") unless flags.empty?
    end

    lib_dirs = lib_dir ? (lib_dir.is_a?(Array) ? lib_dir.dup : lib_dir.split(File::PATH_SEPARATOR)) : []
    if defaults
      lib_dirs.concat(defaults.map { |held| "#{held}/lib" })
      lib_dir = ([lib_dir] + lib_dirs).compact.join(File::PATH_SEPARATOR)
    end
    $LIBPATH = lib_dirs | $LIBPATH

    $config_dirs[key] = [include_dir, lib_dir]
  end

  # Print what is being checked for, run the check, and print its answer.
  def checking_for(subject, format = nil)
    message "%s", "checking for #{subject}... "
    answer = yield
    message "%s\n", format ? format % answer : (answer ? "yes" : "no")
    answer
  end

  def message(*parts)
    return if Logging.quiet && !$VERBOSE

    printf(*parts)
    $stdout.flush
  end

  # What a check says it looks for: the target, the headers it looks in, and
  # the options it compiles with.
  def checking_message(target, place = nil, options = nil)
    [["in", place], ["with", options]].inject(+"#{target}") do |said, (word, noun)|
      if noun
        noun = noun.is_a?(Array) ? noun.join(",") : noun.to_s
        unless noun.empty?
          said << " #{word} " unless said.empty?
          said << noun
        end
      end
      said
    end
  end

  def cpp_include(header)
    return "" unless header
    [*header].map { |held| held.is_a?(String) ? "#include <#{held}>\n" : held }.join
  end

  def append_library(libs, lib)
    format(LIBARG, lib) + " " + libs
  end

  # Whether the source compiles to an object file.
  # Whether the source compiles to an object file. With `werror`, a warning
  # fails the check as an error does.
  def try_compile(source, options = "", werror: nil, **_options)
    options = [options, config_string("WERRORFLAG")].compact.join(" ") if werror
    build_check(source) do |file|
      run_quietly("#{compiler} #{include_flags} #{preprocessor_flags} #{$CFLAGS} #{options} -c #{file} -o conftest.o", werror: werror)
    end
  end

  alias try_header try_compile

  # Whether the source links against the `rb_*` functions the metorex
  # binary exports, as an extension does. The source links into a shared
  # object that leaves those to be found when it is loaded, as the
  # extension's own link does, and counts only if this interpreter loads it
  # with every symbol it names resolved.
  def try_link(source, options = "", ldflags: nil, **_options)
    build_check(source) do |file|
      library_paths = $LIBPATH.map { |held| "-L#{held}" }.join(" ")
      linked = run_quietly("#{compiler} #{link_output} #{include_flags} #{preprocessor_flags} #{$CFLAGS} #{file} #{library_paths} #{$LDFLAGS} #{ldflags} #{options} #{$LIBS}")
      linked && loads_with_every_symbol?(darwin? ? "conftest.bundle" : "conftest.so")
    end
  end

  # Whether `func` links, taken by its address and then called.
  def try_func(func, libs, headers = nil, options = "")
    headers = cpp_include(headers)
    prepared = +""
    case func
    when /^&/
      declared = proc { |name| "const volatile void *#{name}" }
    when /\)$/
      strings = []
      call = func.gsub(/""/) do
        named = "s#{strings.size + 1}"
        strings << named
        named
      end
      prepared << "char " << strings.map { |named| %(#{named}[1024] = "") }.join(", ") << "; " unless strings.empty?
    when nil
      call = ""
    else
      call = "#{func}()"
      declared = proc { |name| "void ((*#{name})())" }
    end
    options = options.to_s.empty? ? libs : "#{options} #{libs}"
    taken = declared && try_link(<<~SOURCE, options)
      #{headers}
      extern int t(void);
      #{MAIN_DOES_NOTHING}
      int t(void) { #{declared["volatile p"]}; p = (#{declared[nil]})#{func}; return !p; }
    SOURCE
    taken || (call && try_link(<<~SOURCE, options))
      #{headers}
      extern int t(void);
      #{MAIN_DOES_NOTHING}
      #{"extern void #{call};" if declared}
      int t(void) { #{prepared}#{call}; return 0; }
    SOURCE
  end

  def try_var(var, headers = nil, options = "")
    try_compile(<<~SOURCE, options)
      #{cpp_include(headers)}
      extern int t(void);
      #{MAIN_DOES_NOTHING}
      int t(void) { const volatile void *volatile p; p = &(&#{var})[0]; return !p; }
    SOURCE
  end

  def try_type(type, headers = nil, options = "")
    return false unless try_compile(<<~SOURCE, options)
      #{cpp_include(headers)}
      typedef #{type} conftest_type;
      int conftestval[sizeof(conftest_type)?1:-1];
    SOURCE
    $defs.push(format("-DHAVE_TYPE_%s", type.tr_cpp))
    true
  end

  def try_const(const, headers = nil, options = "")
    const, type = *const
    return false unless try_compile(<<~SOURCE, options)
      #{cpp_include(headers)}
      typedef #{type || 'int'} conftest_type;
      conftest_type conftestval = #{type ? '' : '(int)'}#{const};
    SOURCE
    $defs.push(format("-DHAVE_CONST_%s", const.tr_cpp))
    true
  end

  def macro_defined?(macro, source, options = "")
    source = source.sub(/[^\n]\z/, "\\&\n")
    try_compile(source + <<~SOURCE, options)
      #ifndef #{macro}
      # error
      |:/ === #{macro} undefined === /:|
      #endif
    SOURCE
  end

  def have_header(header, preheaders = nil, options = "")
    dir_config(header[/.*?(?=\/)|.*?(?=\.)/])
    checking_for header do
      if try_header(cpp_include(preheaders) + cpp_include(header), options)
        $defs.push(format("-DHAVE_%s", header.tr_cpp))
        true
      else
        false
      end
    end
  end

  def have_library(lib, func = nil, headers = nil, options = "")
    dir_config(lib)
    lib = with_config(lib + "lib", lib)
    checking_for checking_message(func && func.funcall_style, LIBARG % lib, options) do
      if COMMON_LIBS.include?(lib)
        true
      else
        libs = append_library($libs, lib)
        if try_func(func, libs, headers, options)
          $libs = libs
          true
        else
          false
        end
      end
    end
  end

  # Whether `header` compiles, here or from one of `paths`, which goes on
  # $INCFLAGS where the header is found there.
  def find_header(header, *paths)
    message = checking_message(header, paths)
    header = cpp_include(header)
    checking_for message do
      if try_header(header)
        true
      else
        found = paths.find do |directory|
          flag = "-I#{directory}"
          try_header(header, flag) && ($INCFLAGS << " " << flag)
        end
        found ? true : false
      end
    end
  end

  # Whether `func` links from `lib`, here or from one of `paths`, which goes
  # on $LDFLAGS where the library is found there.
  def find_library(lib, func, *paths)
    dir_config(lib)
    lib = with_config(lib + "lib", lib)
    paths = paths.flat_map { |held| held.split(File::PATH_SEPARATOR) }
    checking_for checking_message(func && func.funcall_style, LIBARG % lib) do
      libs = append_library($libs, lib)
      linked = ([nil] + paths).find do |directory|
        flag = directory ? "-L#{directory}" : ""
        next false unless try_func(func, libs, nil, flag)

        $LDFLAGS << " " << flag if directory
        true
      end
      $libs = libs if linked
      linked ? true : false
    end
  end

  # Append each flag the preprocessor accepts to $CPPFLAGS.
  def append_cppflags(flags, **_options)
    Array(flags).each do |flag|
      if checking_for("whether #{flag} is accepted as CPPFLAGS") { try_cppflags(flag) }
        $CPPFLAGS << " " << flag
      end
    end
  end

  def try_cppflags(flags, **_options)
    try_compile(MAIN_DOES_NOTHING, flags, werror: true)
  end

  def have_func(func, headers = nil, options = "")
    checking_for checking_message(func.funcall_style, headers, options) do
      if try_func(func, $libs, headers, options)
        $defs << "-DHAVE_#{func.sans_arguments.tr_cpp}"
        true
      else
        false
      end
    end
  end

  def have_var(var, headers = nil, options = "")
    checking_for checking_message(var, headers, options) do
      if try_var(var, headers, options)
        $defs.push(format("-DHAVE_%s", var.tr_cpp))
        true
      else
        false
      end
    end
  end

  def have_type(type, headers = nil, options = "")
    checking_for checking_message(type, headers, options) do
      try_type(type, headers, options)
    end
  end

  def have_macro(macro, headers = nil, options = "")
    checking_for checking_message(macro, headers, options) do
      macro_defined?(macro, cpp_include(headers), options)
    end
  end

  def have_struct_member(type, member, headers = nil, options = "")
    checking_for checking_message("#{type}.#{member}", headers) do
      if try_compile(<<~SOURCE, options)
        #{cpp_include(headers)}
        int s = (char *)&((#{type}*)0)->#{member} - (char *)0;
        #{MAIN_DOES_NOTHING}
      SOURCE
        $defs.push(format("-DHAVE_%s_%s", type.tr_cpp, member.tr_cpp))
        $defs.push(format("-DHAVE_ST_%s", member.tr_cpp))
        true
      else
        false
      end
    end
  end

  def have_const(const, headers = nil, options = "")
    checking_for checking_message([*const].compact.join(" "), headers, options) do
      try_const(const, headers, options)
    end
  end

  def create_makefile(target, srcprefix = nil)
    # The sources sit under $(srcdir), the directory the extconf.rb was run
    # from, unless the extconf.rb named them in $srcs. The objects and the
    # library are built in the current directory. Each rule names its source
    # by path, and no VPATH is written, so objects an earlier build left in
    # the source directory are not taken for this build's.
    source_directory = srcprefix ? File.join($srcdir, srcprefix) : $srcdir
    sources = $srcs ? $srcs.map { |source| source_path(source, source_directory) } : Dir.glob(File.join(source_directory, "*.c")).sort
    objects = $objs || sources.map { |source| File.basename(source, ".*") + ".o" }
    dlext = RbConfig::CONFIG["DLEXT"]
    link_flags = if RbConfig::CONFIG["host_os"].include?("darwin")
      "-bundle"
    else
      "-shared"
    end
    library_paths = $LIBPATH.map { |held| "-L#{held}" }.join(" ")
    libraries = [$libs, $LIBS].map(&:strip).reject(&:empty?).join(" ")

    # `create_makefile "debug/debug"` builds debug.so, installed under debug/.
    target_prefix, target = target.include?("/") ? ["/" + File.dirname(target), File.basename(target)] : ["", target]
    makefile = +""
    makefile << "target_prefix = #{target_prefix}\n"
    makefile << "srcdir = #{$srcdir}\n"
    makefile << "hdrdir = #{RbConfig::CONFIG["rubyhdrdir"]}\n"
    makefile << "arch_hdrdir = $(hdrdir)\n"
    makefile << "CC = #{RbConfig::CONFIG["CC"]}\n"
    makefile << "optflags = #{CONFIG["optflags"]}\n"
    makefile << "debugflags = #{CONFIG["debugflags"]}\n"
    makefile << "warnflags = #{CONFIG["warnflags"]}\n"
    makefile << "CFLAGS = -fPIC $(optflags) $(debugflags) $(warnflags) #{$CFLAGS}\n"
    makefile << "INCFLAGS = -I. #{$INCFLAGS}\n"
    makefile << "RUBY_EXTCONF_H = #{$extconf_h}\n"
    prefix_include = srcprefix ? ["-I#{source_directory}"] : []
    definitions = $extconf_h ? ["-DRUBY_EXTCONF_H=\\\"$(RUBY_EXTCONF_H)\\\""] : $defs
    makefile << "CPPFLAGS = $(INCFLAGS) #{[*prefix_include, *definitions, preprocessor_flags].join(" ")}".rstrip << "\n"
    makefile << "DLDFLAGS = #{[link_flags, $DLDFLAGS, CONFIG["EXTDLDFLAGS"], library_paths, $LDFLAGS].map(&:strip).reject(&:empty?).join(" ")}\n"
    makefile << "LIBS = #{libraries}\n"
    makefile << "DLLIB = #{target}.#{dlext}\n"
    makefile << "OBJS = #{objects.join(" ")}\n"
    makefile << "\n"
    makefile << "all: $(DLLIB)\n"
    makefile << "\n"
    makefile << "$(DLLIB): $(OBJS)\n"
    makefile << "\t$(CC) $(DLDFLAGS) -o $@ $(OBJS) $(LIBS)\n"
    # An object is made again when the header the checks wrote changes, as
    # it does when extconf.rb runs again.
    made_from = $extconf_h ? " #{$extconf_h}" : ""
    sources.zip(objects).each do |source, object|
      makefile << "\n"
      makefile << "#{object}: #{source}#{made_from}\n"
      makefile << "\t$(CC) $(CPPFLAGS) $(CFLAGS) -c #{source} -o #{object}\n"
    end
    makefile << "\n"
    makefile << "clean:\n"
    makefile << "\trm -f $(OBJS) $(DLLIB)#{" #{$cleanfiles.join(" ")}" unless $cleanfiles.empty?}\n"
    message "creating Makefile\n"
    File.write("Makefile", makefile)
    true
  end

  private

  def compiler
    RbConfig::CONFIG["CC"]
  end

  def darwin?
    RbConfig::CONFIG["host_os"].include?("darwin")
  end

  def link_output
    if darwin?
      "-bundle -Wl,-undefined,dynamic_lookup -o conftest.bundle"
    else
      "-shared -fPIC -o conftest.so"
    end
  end

  def loads_with_every_symbol?(library)
    require "fiddle"
    Fiddle::Handle.new(File.expand_path(library), Fiddle::RTLD_NOW).close
    true
  rescue Fiddle::DLError
    false
  end

  # A source $srcs names by a relative path is found in the first directory
  # of $VPATH, or the source directory, that holds it.
  def source_path(source, source_directory)
    return source if File.expand_path(source) == source

    directories = [source_directory, *$VPATH.map { |directory| directory.gsub("$(srcdir)", $srcdir) }]
    directories.map { |directory| File.join(directory, source) }.find { |path| File.exist?(path) } ||
      File.join(source_directory, source)
  end

  # The headers' directory and $INCFLAGS, with the Makefile variables
  # $INCFLAGS names put in place, since a check runs the compiler directly.
  def include_flags
    included = $INCFLAGS.gsub("$(hdrdir)", RbConfig::CONFIG["rubyhdrdir"]).gsub("$(srcdir)", $srcdir)
    "-I#{RbConfig::CONFIG["rubyhdrdir"]} #{included.gsub(/\$\(\w+\)/, "")}"
  end

  # $CPPFLAGS with the Makefile's own variables left out, since a check runs
  # the compiler directly.
  def preprocessor_flags
    $CPPFLAGS.gsub(/\$\(\w+\)/, "").split.join(" ")
  end

  # Write the check's source, with the headers every check starts with, to
  # a directory of its own and answer what the block does with it.
  def build_check(source)
    Dir.mktmpdir("mkmf") do |directory|
      Dir.chdir(directory) do
        File.write("conftest.c", "#{COMMON_HEADERS}\n#{source}")
        yield "conftest.c"
      end
    end
  end

  # Whether the command succeeds, and with `werror` prints nothing as well.
  def run_quietly(command, werror: nil)
    return (system(command, out: File::NULL, err: File::NULL) ? true : false) unless werror

    output = IO.popen(command, err: [:child, :out], &:read)
    $?.success? && output.empty?
  end
end

# The command line's `--name=value` words, as `with_config` reads them.
$configure_args = {}
ARGV.each do |word|
  name, value = word.split("=", 2)
  next unless name
  name = name.tr("_", "-")
  if name.sub!(/\A(?!--)/, "--")
    next unless value
    name.downcase!
  end
  $configure_args[name] = value || true
end

$arg_config = []
$config_dirs = {}
$srcdir = $configure_args["--srcdir"] || File.dirname($0)
$INCFLAGS = "-I$(hdrdir) -I$(srcdir)"
$VPATH = %w[$(srcdir)]
$srcs = nil
$objs = nil
$cleanfiles = []
$distcleanfiles = []
$headers = []
$defs = []
$libs = ""
$LIBPATH = []
$CFLAGS = ""
$CPPFLAGS = ""
$LDFLAGS = ""
$LIBS = ""
$extconf_h = nil

include MakeMakefile

$DLDFLAGS = with_config("dldflags", arg_config("DLDFLAGS", CONFIG["DLDFLAGS"])).dup
