require "tmpdir"

def spot_in(path)
  load path
rescue NameError => error
  location = error.backtrace_locations.find { |held| held.path == path }
  spot = ErrorHighlight.spot(error, backtrace_location: location)
  spot&.values_at(:first_lineno, :first_column, :last_column)
end

Dir.mktmpdir do |dir|
  path = File.join(dir, "parcel.rb")
  File.write(path, "weight = 3\nweight.nope\n")
  p spot_in(path)
  p spot_in(path)
  File.write(path, "weight = 3\n\n  weight.missing_method\n")
  p spot_in(path)
end
