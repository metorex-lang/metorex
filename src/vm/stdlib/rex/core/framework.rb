module Rex
  module Core
    FRAMEWORK_FILE_PREFIX = "<metorex>/rex"

    # Where Rex's own files sit when they are loaded from a directory rather
    # than from the binary.
    FRAMEWORK_ROOT = File.dirname(File.dirname(__FILE__))

    # Whether a caller location is in Rex's own code, by the path it was
    # loaded as or by the file that path resolves to.
    def self.framework_location?(location)
      framework_file?(location.path) || framework_file?(location.absolute_path)
    end

    def self.framework_file?(path)
      path = path.to_s
      path.start_with?(FRAMEWORK_FILE_PREFIX) ||
        (FRAMEWORK_ROOT.start_with?("/") && (path.start_with?(FRAMEWORK_ROOT + "/") || path == FRAMEWORK_ROOT + ".rb"))
    end

    # The first frame of the caller's stack outside Rex, as a deprecation
    # names where it was called from.
    def self.first_non_rex_line
      caller_locations.find { |location| !framework_location?(location) && !location.path.to_s.start_with?("<internal:") }&.to_s
    end
  end

  # Finds the first line of a backtrace outside the RSpec libraries, Rex's
  # own frames among them.
  class CallerFilter
    RSPEC_LIBS = %w[
      core
      mocks
      expectations
      support
      matchers
      rails
    ].freeze

    ADDITIONAL_TOP_LEVEL_FILES = %w[autorun].freeze

    # Rex's own files, and the frame `metorex spec` runs the suite from.
    REX_PATTERNS = [%r{\A<metorex>/}, /\Ametorex spec:/] +
                   (Core::FRAMEWORK_ROOT.start_with?("/") ? [%r{\A#{Regexp.escape(Core::FRAMEWORK_ROOT)}(?:/|\.rb)}] : [])

    LIB_REGEX = Regexp.union(%r{/lib/rspec/(#{(RSPEC_LIBS + ADDITIONAL_TOP_LEVEL_FILES).join('|')})(\.rb|/)}, *REX_PATTERNS)

    IGNORE_REGEX = Regexp.union(LIB_REGEX, "rubygems/core_ext/kernel_require.rb", "<internal:",
                                %r{/lib/ruby/[^/]+/bundled_gems\.rb})

    def self.first_non_rspec_line(skip_frames = 3, increment = 5)
      loop do
        stack = caller_locations(skip_frames, increment)
        raise "No non-lib lines in stack" unless stack

        line = stack.find { |location| location.path !~ IGNORE_REGEX }
        return line.to_s if line

        skip_frames += increment
        increment *= 2
      end
    end
  end
end
