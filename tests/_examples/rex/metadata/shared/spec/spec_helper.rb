require "support/collection_examples"

RSpec.configure do |config|
  config.include_context "with a logger", :logged
end
