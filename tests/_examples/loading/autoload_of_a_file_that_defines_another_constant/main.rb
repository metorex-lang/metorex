# An autoload whose file defines some other constant raises NameError, and
# afterwards the name is neither a constant nor an autoload, while the file
# is listed as loaded and is not run again. Taking the file back out of the
# list and registering the autoload again makes the name an autoload once
# more, as reloading code does.
$runs = 0
missing = File.join(__dir__, "defines_another.rb")
Object.autoload(:Missing, missing)
p [Object.constants.include?(:Missing), Object.const_defined?(:Missing, false), Object.autoload?(:Missing) == missing]
begin
  Missing
rescue NameError => error
  p [error.class, error.name]
end
p [Object.constants.include?(:Missing), Object.const_defined?(:Missing, false), Object.autoload?(:Missing)]
p $LOADED_FEATURES.any? { |feature| feature.end_with?("/defines_another.rb") }
p defined?(Missing)
p Object.const_source_location(:Missing)
begin
  Missing
rescue NameError => error
  p [error.class, error.name]
end
p $runs
p Object.autoload?(:Missing, false)
begin
  Object.const_get(:Missing, false)
rescue NameError => error
  p [:const_get, error.name]
end
begin
  Object.send(:remove_const, :Missing)
rescue NameError => error
  p [:remove_const, error.message]
end
p $runs
path = File.join(__dir__, "defines_another.rb")
$LOADED_FEATURES.reject! { |feature| feature == path }
Object.autoload(:Missing, path)
p [Object.constants.include?(:Missing), Object.const_defined?(:Missing, false), Object.autoload?(:Missing) == path]

# A Kernel#require that raises after the file has loaded, as Zeitwerk's does
# when the file defines the wrong constant, leaves the file loaded and the
# name no autoload.
module Kernel
  alias_method :original_require, :require

  def require(path)
    loaded = original_require(path)
    raise NameError.new("expected #{path} to define Checked", :Checked) if path.end_with?("checked.rb")

    loaded
  end
end
checked = File.join(__dir__, "checked.rb")
Object.autoload(:Checked, checked)
begin
  Checked
rescue NameError => error
  p [error.class, error.name]
end
p [Object.constants.include?(:Checked), Object.const_defined?(:Checked, false), Object.autoload?(:Checked) == checked]
p $LOADED_FEATURES.include?(checked)
p $runs
