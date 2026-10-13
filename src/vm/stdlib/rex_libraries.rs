// Rex, the spec framework `metorex spec` runs, and the RSpec names that
// load it.

/// What `require "rspec"` and its siblings load: Rex, which RSpec names.
const LOADS_REX: &str = "require \"rex\"\n";

pub(super) const REX_LIBRARIES: &[(&str, &str)] = &[
    ("rex", include_str!("rex.rb")),
    ("rex/support", include_str!("rex/support.rb")),
    ("rex/support/source", include_str!("rex/support/source.rb")),
    (
        "rex/expectations/block_snippet_extractor",
        include_str!("rex/expectations/block_snippet_extractor.rb"),
    ),
    ("rex/core/bisect", include_str!("rex/core/bisect.rb")),
    (
        "rex/core/configuration",
        include_str!("rex/core/configuration.rb"),
    ),
    (
        "rex/core/deprecations",
        include_str!("rex/core/deprecations.rb"),
    ),
    ("rex/core/dsl", include_str!("rex/core/dsl.rb")),
    ("rex/core/example", include_str!("rex/core/example.rb")),
    (
        "rex/core/example_group",
        include_str!("rex/core/example_group.rb"),
    ),
    (
        "rex/core/formatters",
        include_str!("rex/core/formatters.rb"),
    ),
    ("rex/core/hooks", include_str!("rex/core/hooks.rb")),
    ("rex/diff", include_str!("rex/diff.rb")),
    ("rex/source", include_str!("rex/source.rb")),
    (
        "rex/core/html_formatter",
        include_str!("rex/core/html_formatter.rb"),
    ),
    (
        "rex/core/json_formatter",
        include_str!("rex/core/json_formatter.rb"),
    ),
    (
        "rex/core/memoized_helpers",
        include_str!("rex/core/memoized_helpers.rb"),
    ),
    ("rex/core/metadata", include_str!("rex/core/metadata.rb")),
    ("rex/core/coverage", include_str!("rex/core/coverage.rb")),
    ("rex/core/parallel", include_str!("rex/core/parallel.rb")),
    ("rex/core/runner", include_str!("rex/core/runner.rb")),
    (
        "rex/core/shared_example_group",
        include_str!("rex/core/shared_example_group.rb"),
    ),
    ("rex/core/world", include_str!("rex/core/world.rb")),
    ("rex/expectations", include_str!("rex/expectations.rb")),
    (
        "rex/expectations/failure_aggregator",
        include_str!("rex/expectations/failure_aggregator.rb"),
    ),
    ("rex/matchers", include_str!("rex/matchers.rb")),
    (
        "rex/matchers/block_snippet",
        include_str!("rex/matchers/block_snippet.rb"),
    ),
    (
        "rex/matchers/differ",
        include_str!("rex/matchers/differ.rb"),
    ),
    ("rex/matchers/dsl", include_str!("rex/matchers/dsl.rb")),
    ("rex/mocks", include_str!("rex/mocks.rb")),
    ("rex/mocks/proxy", include_str!("rex/mocks/proxy.rb")),
    ("rex/mocks/targets", include_str!("rex/mocks/targets.rb")),
    (
        "rex/mocks/any_instance",
        include_str!("rex/mocks/any_instance.rb"),
    ),
    (
        "rex/mocks/mutate_const",
        include_str!("rex/mocks/mutate_const.rb"),
    ),
    (
        "rex/mocks/configuration",
        include_str!("rex/mocks/configuration.rb"),
    ),
    (
        "rex/mocks/verifying",
        include_str!("rex/mocks/verifying.rb"),
    ),
    (
        "rex/matchers/extras",
        include_str!("rex/matchers/extras.rb"),
    ),
    (
        "rex/matchers/fail_matchers",
        include_str!("rex/matchers/fail_matchers.rb"),
    ),
    ("rspec", LOADS_REX),
    ("rspec/core", LOADS_REX),
    ("rspec/expectations", LOADS_REX),
    ("rspec/mocks", LOADS_REX),
    ("rspec/autorun", LOADS_REX),
    ("rspec/matchers", LOADS_REX),
    ("rspec/support", LOADS_REX),
    ("rspec/support/ruby_features", LOADS_REX),
    ("rspec/support/recursive_const_methods", LOADS_REX),
    (
        "rspec/expectations/block_snippet_extractor",
        "require \"rex/expectations/block_snippet_extractor\"\n",
    ),
    (
        "rspec/matchers/fail_matchers",
        "require \"rex/matchers/fail_matchers\"\n",
    ),
    ("rspec/core/formatters", LOADS_REX),
    ("rspec/core/formatters/base_formatter", LOADS_REX),
    ("rspec/core/formatters/base_text_formatter", LOADS_REX),
    ("rspec/core/formatters/console_codes", LOADS_REX),
    ("rspec/core/formatters/progress_formatter", LOADS_REX),
    ("rspec/core/formatters/documentation_formatter", LOADS_REX),
    (
        "rspec/core/formatters/json_formatter",
        "require \"rex\"\nrequire \"rex/core/json_formatter\"\n",
    ),
    (
        "rspec/core/formatters/html_formatter",
        "require \"rex\"\nrequire \"rex/core/html_formatter\"\n",
    ),
];
