require_relative "library"

begin
  Settings.new.alias_name(:feature)
rescue NoMethodError => error
  puts error.message
  puts error.backtrace.first(2).map { |line| line.sub(%r{\A[^:]*/}, "") }
end

begin
  Settings.new.missing_name
rescue NoMethodError => error
  puts error.backtrace.first(2).map { |line| line.sub(%r{\A[^:]*/}, "") }
end
