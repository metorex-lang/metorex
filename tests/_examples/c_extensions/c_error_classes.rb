# Two error classes a C extension defines under StandardError are told
# apart by ===, rescue, and case, for an error the extension raises.
require "tmpdir"
require_relative "build_helper"

directory = Dir.mktmpdir
require(build_extension("c_error_classes.c", "c_error_classes", directory))

begin
  CErrorClasses.parse("text")
rescue CErrorClasses::Secure::SecureError
  puts "rescued as SecureError"
rescue CErrorClasses::ParseError => error
  p error.class
  p error.message
  p CErrorClasses::ParseError === error
  p CErrorClasses::Secure::SecureError === error
  p error.is_a?(CErrorClasses::Secure::SecureError)
  p(case error
    when CErrorClasses::Secure::SecureError then :secure
    when CErrorClasses::ParseError then :parse
    end)
end
