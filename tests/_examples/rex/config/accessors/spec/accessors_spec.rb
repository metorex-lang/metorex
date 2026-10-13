SETTINGS = ["backtrace_exclusion_patterns", "backtrace_inclusion_patterns", "bisect_runner", "color", "color_mode", "default_color", "default_formatter", "default_path", "deprecation_stream", "detail_color", "disable_monkey_patching", "drb", "drb_port", "dry_run", "error_exit_code", "error_stream", "example_status_persistence_file_path", "exclude_pattern", "exclusion_filter", "fail_fast", "fail_if_no_examples", "failure_color", "failure_exit_code", "filter", "fixed_color", "full_cause_backtrace", "full_description", "inclusion_filter", "libs", "max_displayed_failure_line_count", "mock_framework", "output_stream", "pattern", "pending_color", "pending_failure_output", "profile_examples", "project_source_dirs", "requires", "run_all_when_everything_filtered", "seed", "shared_context_metadata_behavior", "silence_filter_announcements", "success_color", "threadsafe", "tty"]

RSpec.describe "the configuration" do
  SETTINGS.each do |name|
    it "reads and writes #{name}" do
      config = RSpec.configuration
      expect([config.respond_to?(name), config.respond_to?("#{name}=")]).to eq([true, true])
    end
  end
end

RSpec.describe RSpec::Core::Pending do
  it "names RSpec's pending constants" do
    expect(described_class.constants.sort).to eq(%i[NOT_YET_IMPLEMENTED NO_REASON_GIVEN PendingExampleFixedError SkipDeclaredInExample])
  end

  it "marks examples pending, skipped, and fixed" do
    expect(%i[mark_fixed! mark_pending! mark_skipped!].all? { |name| described_class.respond_to?(name) }).to be(true)
  end
end
