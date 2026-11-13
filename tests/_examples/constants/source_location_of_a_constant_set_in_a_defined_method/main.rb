# A constant assigned in a method made with define_method records the file
# the block was written in, not the file that called the method, also when
# a macro hands the block on, a method in another file requires the file,
# and the methods run through `send` from at_exit, as Minitest runs tests.
require_relative "macro"
require_relative "runner"
Dir.chdir(__dir__) { Runner.load_files(["definitions"]) }
at_exit do
  Runner.run(Definitions.new, %i[assign_through_define_method assign_through_macro])
  %i[SET_IN_BLOCK SET_IN_MACRO].each do |name|
    file, line = Object.const_source_location(name)
    p [name, File.basename(file), line]
  end
end
