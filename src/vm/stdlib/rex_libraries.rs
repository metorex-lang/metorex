// Rex, the spec framework `metorex spec` runs, and the RSpec names that
// load it.

/// What `require "rspec"` and its siblings load: Rex, which RSpec names.
const LOADS_REX: &str = "require \"rex\"\n";

/// The libraries Rex carries, by the path `require` names.
pub(super) const REX_LIBRARIES: &[(&str, &str)] = &[
    ("rex", carried_source!("rex.rb")),
    (
        "rex/core/backtrace_formatter",
        carried_source!("rex/core/backtrace_formatter.rb"),
    ),
    (
        "rex/core/bisect/coordinator",
        carried_source!("rex/core/bisect/coordinator.rb"),
    ),
    (
        "rex/core/bisect/example_minimizer",
        carried_source!("rex/core/bisect/example_minimizer.rb"),
    ),
    (
        "rex/core/bisect/fork_runner",
        carried_source!("rex/core/bisect/fork_runner.rb"),
    ),
    (
        "rex/core/bisect/server",
        carried_source!("rex/core/bisect/server.rb"),
    ),
    (
        "rex/core/bisect/shell_command",
        carried_source!("rex/core/bisect/shell_command.rb"),
    ),
    (
        "rex/core/bisect/shell_runner",
        carried_source!("rex/core/bisect/shell_runner.rb"),
    ),
    (
        "rex/core/bisect/utilities",
        carried_source!("rex/core/bisect/utilities.rb"),
    ),
    (
        "rex/core/configuration",
        carried_source!("rex/core/configuration.rb"),
    ),
    (
        "rex/core/configuration_options",
        carried_source!("rex/core/configuration_options.rb"),
    ),
    ("rex/core/coverage", carried_source!("rex/core/coverage.rb")),
    (
        "rex/core/did_you_mean",
        carried_source!("rex/core/did_you_mean.rb"),
    ),
    ("rex/core/drb", carried_source!("rex/core/drb.rb")),
    ("rex/core/dsl", carried_source!("rex/core/dsl.rb")),
    ("rex/core/example", carried_source!("rex/core/example.rb")),
    (
        "rex/core/example_group",
        carried_source!("rex/core/example_group.rb"),
    ),
    (
        "rex/core/example_status_persister",
        carried_source!("rex/core/example_status_persister.rb"),
    ),
    (
        "rex/core/fast_paths",
        carried_source!("rex/core/fast_paths.rb"),
    ),
    (
        "rex/core/filter_manager",
        carried_source!("rex/core/filter_manager.rb"),
    ),
    ("rex/core/flat_map", carried_source!("rex/core/flat_map.rb")),
    (
        "rex/core/formatters",
        carried_source!("rex/core/formatters.rb"),
    ),
    (
        "rex/core/formatters/base_bisect_formatter",
        carried_source!("rex/core/formatters/base_bisect_formatter.rb"),
    ),
    (
        "rex/core/formatters/base_formatter",
        carried_source!("rex/core/formatters/base_formatter.rb"),
    ),
    (
        "rex/core/formatters/base_text_formatter",
        carried_source!("rex/core/formatters/base_text_formatter.rb"),
    ),
    (
        "rex/core/formatters/bisect_drb_formatter",
        carried_source!("rex/core/formatters/bisect_drb_formatter.rb"),
    ),
    (
        "rex/core/formatters/bisect_progress_formatter",
        carried_source!("rex/core/formatters/bisect_progress_formatter.rb"),
    ),
    (
        "rex/core/formatters/console_codes",
        carried_source!("rex/core/formatters/console_codes.rb"),
    ),
    (
        "rex/core/formatters/deprecation_formatter",
        carried_source!("rex/core/formatters/deprecation_formatter.rb"),
    ),
    (
        "rex/core/formatters/documentation_formatter",
        carried_source!("rex/core/formatters/documentation_formatter.rb"),
    ),
    (
        "rex/core/formatters/exception_presenter",
        carried_source!("rex/core/formatters/exception_presenter.rb"),
    ),
    (
        "rex/core/formatters/failure_list_formatter",
        carried_source!("rex/core/formatters/failure_list_formatter.rb"),
    ),
    (
        "rex/core/formatters/fallback_message_formatter",
        carried_source!("rex/core/formatters/fallback_message_formatter.rb"),
    ),
    (
        "rex/core/formatters/helpers",
        carried_source!("rex/core/formatters/helpers.rb"),
    ),
    (
        "rex/core/formatters/html_formatter",
        carried_source!("rex/core/formatters/html_formatter.rb"),
    ),
    (
        "rex/core/formatters/html_printer",
        carried_source!("rex/core/formatters/html_printer.rb"),
    ),
    (
        "rex/core/formatters/html_snippet_extractor",
        carried_source!("rex/core/formatters/html_snippet_extractor.rb"),
    ),
    (
        "rex/core/formatters/json_formatter",
        carried_source!("rex/core/formatters/json_formatter.rb"),
    ),
    (
        "rex/core/formatters/profile_formatter",
        carried_source!("rex/core/formatters/profile_formatter.rb"),
    ),
    (
        "rex/core/formatters/progress_formatter",
        carried_source!("rex/core/formatters/progress_formatter.rb"),
    ),
    (
        "rex/core/formatters/protocol",
        carried_source!("rex/core/formatters/protocol.rb"),
    ),
    (
        "rex/core/formatters/snippet_extractor",
        carried_source!("rex/core/formatters/snippet_extractor.rb"),
    ),
    (
        "rex/core/formatters/syntax_highlighter",
        carried_source!("rex/core/formatters/syntax_highlighter.rb"),
    ),
    (
        "rex/core/framework",
        carried_source!("rex/core/framework.rb"),
    ),
    ("rex/core/hooks", carried_source!("rex/core/hooks.rb")),
    (
        "rex/core/invocations",
        carried_source!("rex/core/invocations.rb"),
    ),
    (
        "rex/core/memoized_helpers",
        carried_source!("rex/core/memoized_helpers.rb"),
    ),
    ("rex/core/metadata", carried_source!("rex/core/metadata.rb")),
    (
        "rex/core/metadata_filter",
        carried_source!("rex/core/metadata_filter.rb"),
    ),
    (
        "rex/core/minitest_assertions_adapter",
        carried_source!("rex/core/minitest_assertions_adapter.rb"),
    ),
    (
        "rex/core/mocking_adapters/flexmock",
        carried_source!("rex/core/mocking_adapters/flexmock.rb"),
    ),
    (
        "rex/core/mocking_adapters/mocha",
        carried_source!("rex/core/mocking_adapters/mocha.rb"),
    ),
    (
        "rex/core/mocking_adapters/null",
        carried_source!("rex/core/mocking_adapters/null.rb"),
    ),
    (
        "rex/core/mocking_adapters/rr",
        carried_source!("rex/core/mocking_adapters/rr.rb"),
    ),
    (
        "rex/core/mocking_adapters/rspec",
        carried_source!("rex/core/mocking_adapters/rspec.rb"),
    ),
    (
        "rex/core/notifications",
        carried_source!("rex/core/notifications.rb"),
    ),
    (
        "rex/core/option_parser",
        carried_source!("rex/core/option_parser.rb"),
    ),
    ("rex/core/ordering", carried_source!("rex/core/ordering.rb")),
    (
        "rex/core/output_wrapper",
        carried_source!("rex/core/output_wrapper.rb"),
    ),
    ("rex/core/parallel", carried_source!("rex/core/parallel.rb")),
    ("rex/core/pending", carried_source!("rex/core/pending.rb")),
    ("rex/core/profiler", carried_source!("rex/core/profiler.rb")),
    (
        "rex/core/project_initializer",
        carried_source!("rex/core/project_initializer.rb"),
    ),
    (
        "rex/core/rake_task",
        carried_source!("rex/core/rake_task.rb"),
    ),
    ("rex/core/reporter", carried_source!("rex/core/reporter.rb")),
    (
        "rex/core/rex_settings",
        carried_source!("rex/core/rex_settings.rb"),
    ),
    (
        "rex/core/ruby_project",
        carried_source!("rex/core/ruby_project.rb"),
    ),
    ("rex/core/runner", carried_source!("rex/core/runner.rb")),
    ("rex/core/sandbox", carried_source!("rex/core/sandbox.rb")),
    ("rex/core/set", carried_source!("rex/core/set.rb")),
    (
        "rex/core/shared_context",
        carried_source!("rex/core/shared_context.rb"),
    ),
    (
        "rex/core/shared_example_group",
        carried_source!("rex/core/shared_example_group.rb"),
    ),
    (
        "rex/core/shell_escape",
        carried_source!("rex/core/shell_escape.rb"),
    ),
    (
        "rex/core/test_unit_assertions_adapter",
        carried_source!("rex/core/test_unit_assertions_adapter.rb"),
    ),
    ("rex/core/version", carried_source!("rex/core/version.rb")),
    ("rex/core/warnings", carried_source!("rex/core/warnings.rb")),
    ("rex/core/world", carried_source!("rex/core/world.rb")),
    (
        "rex/expectation_additions",
        carried_source!("rex/expectation_additions.rb"),
    ),
    ("rex/source", carried_source!("rex/source.rb")),
    (
        "rspec/core/backtrace_formatter",
        "require \"rex\"\nrequire \"rex/core/backtrace_formatter\"\n",
    ),
    (
        "rspec/core/bisect/coordinator",
        "require \"rex\"\nrequire \"rex/core/bisect/coordinator\"\n",
    ),
    (
        "rspec/core/bisect/example_minimizer",
        "require \"rex\"\nrequire \"rex/core/bisect/example_minimizer\"\n",
    ),
    (
        "rspec/core/bisect/fork_runner",
        "require \"rex\"\nrequire \"rex/core/bisect/fork_runner\"\n",
    ),
    (
        "rspec/core/bisect/server",
        "require \"rex\"\nrequire \"rex/core/bisect/server\"\n",
    ),
    (
        "rspec/core/bisect/shell_command",
        "require \"rex\"\nrequire \"rex/core/bisect/shell_command\"\n",
    ),
    (
        "rspec/core/bisect/shell_runner",
        "require \"rex\"\nrequire \"rex/core/bisect/shell_runner\"\n",
    ),
    (
        "rspec/core/bisect/utilities",
        "require \"rex\"\nrequire \"rex/core/bisect/utilities\"\n",
    ),
    (
        "rspec/core/configuration",
        "require \"rex\"\nrequire \"rex/core/configuration\"\n",
    ),
    (
        "rspec/core/configuration_options",
        "require \"rex\"\nrequire \"rex/core/configuration_options\"\n",
    ),
    (
        "rspec/core/did_you_mean",
        "require \"rex\"\nrequire \"rex/core/did_you_mean\"\n",
    ),
    (
        "rspec/core/drb",
        "require \"rex\"\nrequire \"rex/core/drb\"\n",
    ),
    (
        "rspec/core/dsl",
        "require \"rex\"\nrequire \"rex/core/dsl\"\n",
    ),
    (
        "rspec/core/example",
        "require \"rex\"\nrequire \"rex/core/example\"\n",
    ),
    (
        "rspec/core/example_group",
        "require \"rex\"\nrequire \"rex/core/example_group\"\n",
    ),
    (
        "rspec/core/example_status_persister",
        "require \"rex\"\nrequire \"rex/core/example_status_persister\"\n",
    ),
    (
        "rspec/core/fast_paths",
        "require \"rex\"\nrequire \"rex/core/fast_paths\"\n",
    ),
    (
        "rspec/core/filter_manager",
        "require \"rex\"\nrequire \"rex/core/filter_manager\"\n",
    ),
    (
        "rspec/core/flat_map",
        "require \"rex\"\nrequire \"rex/core/flat_map\"\n",
    ),
    (
        "rspec/core/formatters",
        "require \"rex\"\nrequire \"rex/core/formatters\"\n",
    ),
    (
        "rspec/core/formatters/base_bisect_formatter",
        "require \"rex\"\nrequire \"rex/core/formatters/base_bisect_formatter\"\n",
    ),
    (
        "rspec/core/formatters/base_formatter",
        "require \"rex\"\nrequire \"rex/core/formatters/base_formatter\"\n",
    ),
    (
        "rspec/core/formatters/base_text_formatter",
        "require \"rex\"\nrequire \"rex/core/formatters/base_text_formatter\"\n",
    ),
    (
        "rspec/core/formatters/bisect_drb_formatter",
        "require \"rex\"\nrequire \"rex/core/formatters/bisect_drb_formatter\"\n",
    ),
    (
        "rspec/core/formatters/bisect_progress_formatter",
        "require \"rex\"\nrequire \"rex/core/formatters/bisect_progress_formatter\"\n",
    ),
    (
        "rspec/core/formatters/console_codes",
        "require \"rex\"\nrequire \"rex/core/formatters/console_codes\"\n",
    ),
    (
        "rspec/core/formatters/deprecation_formatter",
        "require \"rex\"\nrequire \"rex/core/formatters/deprecation_formatter\"\n",
    ),
    (
        "rspec/core/formatters/documentation_formatter",
        "require \"rex\"\nrequire \"rex/core/formatters/documentation_formatter\"\n",
    ),
    (
        "rspec/core/formatters/exception_presenter",
        "require \"rex\"\nrequire \"rex/core/formatters/exception_presenter\"\n",
    ),
    (
        "rspec/core/formatters/failure_list_formatter",
        "require \"rex\"\nrequire \"rex/core/formatters/failure_list_formatter\"\n",
    ),
    (
        "rspec/core/formatters/fallback_message_formatter",
        "require \"rex\"\nrequire \"rex/core/formatters/fallback_message_formatter\"\n",
    ),
    (
        "rspec/core/formatters/helpers",
        "require \"rex\"\nrequire \"rex/core/formatters/helpers\"\n",
    ),
    (
        "rspec/core/formatters/html_formatter",
        "require \"rex\"\nrequire \"rex/core/formatters/html_formatter\"\n",
    ),
    (
        "rspec/core/formatters/html_printer",
        "require \"rex\"\nrequire \"rex/core/formatters/html_printer\"\n",
    ),
    (
        "rspec/core/formatters/html_snippet_extractor",
        "require \"rex\"\nrequire \"rex/core/formatters/html_snippet_extractor\"\n",
    ),
    (
        "rspec/core/formatters/json_formatter",
        "require \"rex\"\nrequire \"rex/core/formatters/json_formatter\"\n",
    ),
    (
        "rspec/core/formatters/profile_formatter",
        "require \"rex\"\nrequire \"rex/core/formatters/profile_formatter\"\n",
    ),
    (
        "rspec/core/formatters/progress_formatter",
        "require \"rex\"\nrequire \"rex/core/formatters/progress_formatter\"\n",
    ),
    (
        "rspec/core/formatters/protocol",
        "require \"rex\"\nrequire \"rex/core/formatters/protocol\"\n",
    ),
    (
        "rspec/core/formatters/snippet_extractor",
        "require \"rex\"\nrequire \"rex/core/formatters/snippet_extractor\"\n",
    ),
    (
        "rspec/core/formatters/syntax_highlighter",
        "require \"rex\"\nrequire \"rex/core/formatters/syntax_highlighter\"\n",
    ),
    (
        "rspec/core/hooks",
        "require \"rex\"\nrequire \"rex/core/hooks\"\n",
    ),
    (
        "rspec/core/invocations",
        "require \"rex\"\nrequire \"rex/core/invocations\"\n",
    ),
    (
        "rspec/core/memoized_helpers",
        "require \"rex\"\nrequire \"rex/core/memoized_helpers\"\n",
    ),
    (
        "rspec/core/metadata",
        "require \"rex\"\nrequire \"rex/core/metadata\"\n",
    ),
    (
        "rspec/core/metadata_filter",
        "require \"rex\"\nrequire \"rex/core/metadata_filter\"\n",
    ),
    (
        "rspec/core/minitest_assertions_adapter",
        "require \"rex\"\nrequire \"rex/core/minitest_assertions_adapter\"\n",
    ),
    (
        "rspec/core/mocking_adapters/flexmock",
        "require \"rex\"\nrequire \"rex/core/mocking_adapters/flexmock\"\n",
    ),
    (
        "rspec/core/mocking_adapters/mocha",
        "require \"rex\"\nrequire \"rex/core/mocking_adapters/mocha\"\n",
    ),
    (
        "rspec/core/mocking_adapters/null",
        "require \"rex\"\nrequire \"rex/core/mocking_adapters/null\"\n",
    ),
    (
        "rspec/core/mocking_adapters/rr",
        "require \"rex\"\nrequire \"rex/core/mocking_adapters/rr\"\n",
    ),
    (
        "rspec/core/mocking_adapters/rspec",
        "require \"rex\"\nrequire \"rex/core/mocking_adapters/rspec\"\n",
    ),
    (
        "rspec/core/notifications",
        "require \"rex\"\nrequire \"rex/core/notifications\"\n",
    ),
    (
        "rspec/core/option_parser",
        "require \"rex\"\nrequire \"rex/core/option_parser\"\n",
    ),
    (
        "rspec/core/ordering",
        "require \"rex\"\nrequire \"rex/core/ordering\"\n",
    ),
    (
        "rspec/core/output_wrapper",
        "require \"rex\"\nrequire \"rex/core/output_wrapper\"\n",
    ),
    (
        "rspec/core/pending",
        "require \"rex\"\nrequire \"rex/core/pending\"\n",
    ),
    (
        "rspec/core/profiler",
        "require \"rex\"\nrequire \"rex/core/profiler\"\n",
    ),
    (
        "rspec/core/project_initializer",
        "require \"rex\"\nrequire \"rex/core/project_initializer\"\n",
    ),
    (
        "rspec/core/rake_task",
        "require \"rex\"\nrequire \"rex/core/rake_task\"\n",
    ),
    (
        "rspec/core/reporter",
        "require \"rex\"\nrequire \"rex/core/reporter\"\n",
    ),
    (
        "rspec/core/rex_settings",
        "require \"rex\"\nrequire \"rex/core/rex_settings\"\n",
    ),
    (
        "rspec/core/ruby_project",
        "require \"rex\"\nrequire \"rex/core/ruby_project\"\n",
    ),
    (
        "rspec/core/runner",
        "require \"rex\"\nrequire \"rex/core/runner\"\n",
    ),
    (
        "rspec/core/sandbox",
        "require \"rex\"\nrequire \"rex/core/sandbox\"\n",
    ),
    (
        "rspec/core/set",
        "require \"rex\"\nrequire \"rex/core/set\"\n",
    ),
    (
        "rspec/core/shared_context",
        "require \"rex\"\nrequire \"rex/core/shared_context\"\n",
    ),
    (
        "rspec/core/shared_example_group",
        "require \"rex\"\nrequire \"rex/core/shared_example_group\"\n",
    ),
    (
        "rspec/core/shell_escape",
        "require \"rex\"\nrequire \"rex/core/shell_escape\"\n",
    ),
    (
        "rspec/core/test_unit_assertions_adapter",
        "require \"rex\"\nrequire \"rex/core/test_unit_assertions_adapter\"\n",
    ),
    (
        "rspec/core/version",
        "require \"rex\"\nrequire \"rex/core/version\"\n",
    ),
    (
        "rspec/core/warnings",
        "require \"rex\"\nrequire \"rex/core/warnings\"\n",
    ),
    (
        "rspec/core/world",
        "require \"rex\"\nrequire \"rex/core/world\"\n",
    ),
    (
        "rex/autoload",
        "require \"rex\"\nRex::Core::Runner.autorun\n",
    ),
    ("rspec", LOADS_REX),
    ("rspec/core", LOADS_REX),
    ("rspec/expectations", LOADS_REX),
    ("rspec/matchers", LOADS_REX),
    ("rspec/support", LOADS_REX),
    ("rspec/support/caller_filter", LOADS_REX),
    ("rspec/mocks", LOADS_REX),
    ("rspec/autorun", "require \"rex/autoload\"\n"),
];
