module Rex
  module Core
    module Metadata
      # `describe "x", :slow, fast: false` carries `{ slow: true, fast: false }`.
      # A trailing Hash and the Symbols before it are metadata.
      def self.build_hash_from(arguments)
        split(arguments).last
      end

      # The arguments that are not metadata, and the metadata Hash.
      def self.split(arguments)
        arguments = arguments.dup
        hash = arguments.last.is_a?(Hash) ? arguments.pop.dup : {}
        hash[arguments.pop] = true while arguments.last.is_a?(Symbol)
        [arguments, hash]
      end

      def self.filter_applies?(filter, metadata)
        filter.all? { |key, value| value_applies?(key, value, metadata) }
      end

      def self.filter_applies_to_any?(filter, metadata)
        filter.any? { |key, value| value_applies?(key, value, metadata) }
      end

      def self.value_applies?(key, value, metadata)
        return false unless metadata.key?(key)

        actual = metadata[key]
        case value
        when Proc
          if value.arity == 2
            value.call(actual, metadata)
          else
            value.call(actual)
          end
        when Regexp
          value.match?(actual.to_s)
        when Hash
          actual.is_a?(Hash) && value.all? { |inner_key, inner_value| value_applies?(inner_key, inner_value, actual) }
        when Array
          Array(actual).any? { |item| value.include?(item) }
        when true
          actual ? true : false
        when false
          !actual
        when Symbol, String
          actual.to_s == value.to_s || (actual.is_a?(Array) && actual.map(&:to_s).include?(value.to_s))
        else
          actual == value
        end
      end

      # The location `describe` or `it` was written at, read from the frame
      # that called it.
      def self.location_from(locations)
        frame = locations.find { |location| !Rex::Core.framework_location?(location) }
        frame ||= locations.first
        [Rex::Core.relative_path(frame.absolute_path || frame.path), frame.lineno]
      end
    end

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

    # A backtrace line with the working directory written as `.`, as RSpec
    # writes it, or nil for the line `-e:1`.
    def self.relative_path_in_line(line)
      line = line.sub(/(\A|\s)#{Regexp.escape(File.expand_path('.'))}(\/|\s|\Z)/, "\\1.\\2")
      line == "-e:1" ? nil : line
    end

    # `./spec/x_spec.rb` for a file under the working directory, the way RSpec
    # prints a path.
    def self.relative_path(path)
      path = path.to_s
      directory = Dir.pwd
      if path.start_with?(directory + "/")
        "./" + path[(directory.length + 1)..]
      else
        path
      end
    end
  end
end
