RSpec.configure do |config|
  config.backtrace_exclusion_patterns << %r{/vendor/}
  config.backtrace_inclusion_patterns << /fake_client/
end
