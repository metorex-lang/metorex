SLOW_EXPECTED = true
AUDIT_LOG = []

RSpec.configure do |config|
  config.define_derived_metadata(file_path: %r{/spec/}) do |metadata|
    metadata[:slow] = true
  end
  config.before(:context, :audited) { AUDIT_LOG << :context }
  config.before(:example, :tracked) { AUDIT_LOG << :example }
end
