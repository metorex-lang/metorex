require "rspec/support/directory_maker"

module Rex::Core::Formatters
  autoload :DocumentationFormatter,   'rspec/core/formatters/documentation_formatter'
  autoload :HtmlFormatter,            'rspec/core/formatters/html_formatter'
  autoload :FallbackMessageFormatter, 'rspec/core/formatters/fallback_message_formatter'
  autoload :ProgressFormatter,        'rspec/core/formatters/progress_formatter'
  autoload :ProfileFormatter,         'rspec/core/formatters/profile_formatter'
  autoload :JsonFormatter,            'rspec/core/formatters/json_formatter'
  autoload :BisectDRbFormatter,       'rspec/core/formatters/bisect_drb_formatter'
  autoload :ExceptionPresenter,       'rspec/core/formatters/exception_presenter'
  autoload :FailureListFormatter,     'rspec/core/formatters/failure_list_formatter'

  def self.register(formatter_class, *notifications)
    Loader.formatters[formatter_class] = notifications
  end

  class Loader
    def self.formatters
      @formatters ||= {}
    end

    def initialize(reporter)
      @formatters = []
      @reporter = reporter
      self.default_formatter = 'progress'
    end

    attr_reader :formatters

    attr_reader :reporter

    attr_accessor :default_formatter

    def prepare_default(output_stream, deprecation_stream)
      reporter.prepare_default(self, output_stream, deprecation_stream)
    end

    def setup_default(output_stream, deprecation_stream)
      add default_formatter, output_stream if @formatters.empty?

      unless @formatters.any? { |formatter| DeprecationFormatter === formatter }
        add DeprecationFormatter, deprecation_stream, output_stream
      end

      unless existing_formatter_implements?(:message)
        add FallbackMessageFormatter, output_stream
      end

      return unless Rex.configuration.profile_examples?
      return if existing_formatter_implements?(:dump_profile)

      add Rex::Core::Formatters::ProfileFormatter, output_stream
    end

    def add(formatter_to_use, *paths)
      if Loader.formatters.key?(formatter_to_use.class)
        register formatter_to_use, notifications_for(formatter_to_use.class)
        return
      end

      formatter_class = find_formatter(formatter_to_use)

      args = paths.map { |p| p.respond_to?(:puts) ? p : open_stream(p) }

      if !Loader.formatters[formatter_class].nil?
        formatter = formatter_class.new(*args)
        register formatter, notifications_for(formatter_class)
      elsif defined?(Rex::LegacyFormatters)
        formatter = Rex::LegacyFormatters.load_formatter formatter_class, *args
        register formatter, formatter.notifications
      else
        call_site = "Formatter added at: #{::Rex::CallerFilter.first_non_rspec_line}"

        Rex.warn_deprecation <<-WARNING.gsub(/\s*\|/, ' ')
          |The #{formatter_class} formatter uses the deprecated formatter
          |interface not supported directly by RSpec 3.
          |
          |To continue to use this formatter you must install the
          |`rspec-legacy_formatters` gem, which provides support
          |for legacy formatters or upgrade the formatter to a
          |compatible version.
          |
          |#{call_site}
        WARNING
      end
    end

  private

    def find_formatter(formatter_to_use)
      built_in_formatter(formatter_to_use) ||
      custom_formatter(formatter_to_use)   ||
      (raise ArgumentError, "Formatter '#{formatter_to_use}' unknown - " \
                            "maybe you meant 'documentation' or 'progress'?.")
    end

    def register(formatter, notifications)
      return if duplicate_formatter_exists?(formatter)
      @reporter.register_listener formatter, *notifications
      @formatters << formatter
      formatter
    end

    def duplicate_formatter_exists?(new_formatter)
      @formatters.any? do |formatter|
        formatter.class == new_formatter.class &&
          has_matching_output?(formatter, new_formatter)
      end
    end

    def has_matching_output?(formatter, new_formatter)
      return true unless formatter.respond_to?(:output) && new_formatter.respond_to?(:output)
      formatter.output == new_formatter.output
    end

    def existing_formatter_implements?(notification)
      @reporter.registered_listeners(notification).any?
    end

    def built_in_formatter(key)
      case key.to_s
      when 'd', 'doc', 'documentation'
        DocumentationFormatter
      when 'h', 'html'
        HtmlFormatter
      when 'p', 'progress'
        ProgressFormatter
      when 'j', 'json'
        JsonFormatter
      when 'bisect-drb'
        BisectDRbFormatter
      when 'f', 'failures'
        FailureListFormatter
      end
    end

    def notifications_for(formatter_class)
      formatter_class.ancestors.inject(::Rex::Core::Set.new) do |notifications, klass|
        notifications.merge Loader.formatters.fetch(klass) { ::Rex::Core::Set.new }
      end
    end

    def custom_formatter(formatter_ref)
      if Class === formatter_ref
        formatter_ref
      elsif string_const?(formatter_ref)
        begin
          formatter_ref.gsub(/^::/, '').split('::').inject(Object) { |a, e| a.const_get e }
        rescue NameError
          require(path_for(formatter_ref)) ? retry : raise
        end
      end
    end

    def string_const?(str)
      str.is_a?(String) && /\A[A-Z][a-zA-Z0-9_:]*\z/ =~ str
    end

    def path_for(const_ref)
      underscore_with_fix_for_non_standard_rspec_naming(const_ref)
    end

    def underscore_with_fix_for_non_standard_rspec_naming(string)
      underscore(string).sub(%r{(^|/)r_spec($|/)}, '\\1rspec\\2')
    end

    def underscore(camel_cased_word)
      word = camel_cased_word.to_s.dup
      word.gsub!(/::/, '/')
      word.gsub!(/([A-Z]+)([A-Z][a-z])/, '\1_\2')
      word.gsub!(/([a-z\d])([A-Z])/, '\1_\2')
      word.tr!("-", "_")
      word.downcase!
      word
    end

    def open_stream(path_or_wrapper)
      if Rex::Core::OutputWrapper === path_or_wrapper
        path_or_wrapper.output = open_stream(path_or_wrapper.output)
        path_or_wrapper
      else
        Rex::Support::DirectoryMaker.mkdir_p(File.dirname(path_or_wrapper))
        File.new(path_or_wrapper, 'w')
      end
    end
  end
end
