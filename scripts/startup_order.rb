# Writes build/startup.order, the functions benchmarks/startup/empty.rb runs,
# named as the release build names them, followed by the functions of core,
# std, and alloc. build.rs hands the file to the macOS linker, which places
# those functions first, so a start reads fewer pages of the binary. The
# standard library is built without coverage counters, so the functions of it
# that run are not known, and it is placed whole.
#
# A build with coverage counters records which functions run. The two builds
# name a function with different crate hashes, so they are matched by the
# names c++filt demangles, which leave the hashes out. The release build's
# names carry hashes of the compiler's version and the package's, so the file
# is rewritten after either changes, and build.rs warns until it is.
#
# usage: ruby scripts/startup_order.rb

require "open3"
require "tmpdir"

ROOT = File.expand_path("..", __dir__)
TARGET = ENV.fetch("CARGO_TARGET_DIR", File.join(ROOT, "target"))
COUNTED_TARGET = File.join(TARGET, "startup-counted")
ORDER = File.join(ROOT, "build", "startup.order")
EMPTY = File.join(ROOT, "benchmarks", "startup", "empty.rb")

def run(*command, **options)
  system(*command, chdir: ROOT, exception: true, **options)
end

def output_of(*command, input: nil)
  output, status = Open3.capture2(*command, stdin_data: input, chdir: ROOT)
  abort "#{command.join(" ")} failed" unless status.success?
  output
end

# The names c++filt demangles `mangled` to, one for each, in order.
def demangled(mangled)
  output_of("c++filt", input: mangled.join("\n")).lines(chomp: true)
end

# The crate a v0 name's path starts in.
def crate_of(symbol)
  root = symbol[/\A__R[A-Za-z]*?Cs[0-9A-Za-z]*_(\d+[A-Za-z_]+)/, 1] or return
  root.sub(/\A\d+/, "")[0, root.to_i]
end

rustc = output_of("rustc", "-vV")
tools = File.join(output_of("rustc", "--print", "sysroot").strip, "lib", "rustlib", rustc[/^host: (.+)$/, 1], "bin")
version = File.read(File.join(ROOT, "Cargo.toml"))[/^version = "(.+)"$/, 1]

ran = Dir.mktmpdir do |counts|
  # The build scripts are counted too, and write their counts where this says.
  run({ "RUSTFLAGS" => "-C instrument-coverage -C symbol-mangling-version=v0",
        "LLVM_PROFILE_FILE" => File.join(counts, "build-%p.profraw") },
      "cargo", "build", "--release", "--target-dir", COUNTED_TARGET)
  run("cargo", "build", "--release")
  run({ "LLVM_PROFILE_FILE" => File.join(counts, "empty.profraw") },
      File.join(COUNTED_TARGET, "release", "metorex"), EMPTY)
  run(File.join(tools, "llvm-profdata"), "merge", "-o", File.join(counts, "empty.profdata"),
      File.join(counts, "empty.profraw"))
  shown = output_of(File.join(tools, "llvm-profdata"), "show", "--all-functions", "--counts",
                    File.join(counts, "empty.profdata"))
  shown.scan(/^  (\S+):\n(?:    .*\n)*?    Function count: (\d+)$/)
       .reject { |_, count| count == "0" }
       .map { |name, _| name.split(";").last }
end

symbols = output_of("nm", File.join(TARGET, "release", "metorex"))
          .lines.map(&:split).select { |fields| fields[1]&.match?(/\A[tT]\z/) }.map(&:last)
# nm prints each name with the underscore macOS puts before every symbol.
by_name = demangled(symbols.map { |symbol| symbol.delete_prefix("_") }).zip(symbols).group_by(&:first)
names = demangled(ran).uniq
placed = names.select { |name| by_name.key?(name) }
counted = placed.flat_map { |name| by_name[name].map(&:last) }

standard = symbols.select { |symbol| %w[core std alloc].include?(crate_of(symbol)) } - counted

File.write(ORDER, "# #{rustc.lines.first.strip} metorex #{version}\n" +
                  (counted + standard).map { |symbol| "#{symbol}\n" }.join)
puts "#{placed.size} of the #{names.size} functions that ran are functions of the release build, " \
     "the rest inlined into them, and #{standard.size} of the standard library's follow them"
