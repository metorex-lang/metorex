require "rubygems"
require "tmpdir"
require "fileutils"
require "rbconfig"

# A gem laid out as digest-crc's is: a Rakefile extension that runs an
# extconf.rb in a subdirectory and leaves the library it builds there, under
# the "ext" require path. It is built in place, as installing it does, and
# marked built where RubyGems keeps the mark for a shared or a static Ruby.
Dir.mktmpdir do |home|
  gem_dir = File.join(home, "gems", "sum_meter-1.0.0")
  rakefile_dir = File.join(gem_dir, "ext", "sum_meter")
  adder_dir = File.join(rakefile_dir, "adder")
  FileUtils.mkdir_p(adder_dir)
  FileUtils.mkdir_p(File.join(gem_dir, "lib"))

  File.write(File.join(rakefile_dir, "Rakefile"), <<~RAKE)
    task :default do
      Dir.chdir("adder") do
        ruby "-S", "extconf.rb"
        sh "make"
      end
    end
  RAKE
  File.write(File.join(adder_dir, "extconf.rb"), "require \"mkmf\"\ncreate_makefile \"adder_ext\"\n")
  File.write(File.join(adder_dir, "adder_ext.c"), <<~C)
    #include <ruby.h>

    static VALUE add(VALUE self, VALUE left, VALUE right) {
      return INT2NUM(NUM2INT(left) + NUM2INT(right));
    }

    void Init_adder_ext(void) {
      rb_define_module_function(rb_define_module("SumMeterNative"), "add", add, 2);
    }
  C
  File.write(File.join(gem_dir, "lib", "sum_meter.rb"), <<~RUBY)
    begin
      require "sum_meter/adder/adder_ext"
    rescue LoadError
    end

    module SumMeter
      def self.engine
        defined?(SumMeterNative) ? :c : :ruby
      end
    end
  RUBY

  spec = Gem::Specification.new do |gem|
    gem.name = "sum_meter"
    gem.version = "1.0.0"
    gem.summary = "Adds two numbers"
    gem.authors = ["Example"]
    gem.extensions = ["ext/sum_meter/Rakefile"]
    gem.require_paths = ["ext", "lib"]
  end
  FileUtils.mkdir_p(File.join(home, "specifications"))
  File.write(File.join(home, "specifications", "sum_meter-1.0.0.gemspec"), spec.to_ruby)

  built = system(Gem.ruby, "-rrake", "-e", "Rake.application.run(ARGV)", chdir: rakefile_dir,
    out: File::NULL, err: File::NULL)
  p built
  [RbConfig::CONFIG["ruby_version"], "#{RbConfig::CONFIG["ruby_version"]}-static"].each do |version|
    marked = File.join(home, "extensions", Gem::Platform.local.to_s, version, "sum_meter-1.0.0")
    FileUtils.mkdir_p(marked)
    File.write(File.join(marked, "gem.build_complete"), "")
  end

  program = "require \"sum_meter\"; p SumMeter.engine; p SumMeterNative.add(2, 3)"
  environment = { "GEM_HOME" => home, "GEM_PATH" => home }
  puts IO.popen(environment, [Gem.ruby, "-e", program], err: [:child, :out], &:read)
end
