HOOK_ORDER = []

RSpec.configure do |config|
  config.alias_example_to :scenario, slow: true
  config.alias_example_group_to :feature, type: :feature
  config.alias_it_behaves_like_to :it_has_behavior, "has behavior:"

  config.before { HOOK_ORDER << :before }
  config.prepend_before { HOOK_ORDER << :prepend_before }
  config.append_before { HOOK_ORDER << :append_before }
  config.after { HOOK_ORDER << :after }
  config.prepend_after { HOOK_ORDER << :prepend_after }
  config.append_after { HOOK_ORDER << :append_after }

  config.add_setting :api_host, default: "localhost"
  config.add_setting :retries

  config.expose_current_running_example_as :current_spec

  config.when_first_matching_example_defined(:database) do
    config.before(:context, :database) { puts "the database is ready" }
  end

  config.format_docstrings { |text| text.tr("_", " ") }
end
