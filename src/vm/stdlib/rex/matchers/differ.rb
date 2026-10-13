require "pp"
require "rex/diff"

module Rex
  module Matchers
    # The diff a failure message ends with when expected and actual differ in
    # a way a reader can scan: multi-line strings, Hashes, and Arrays.
    module Differ
      CONTEXT_LINES = 3

      def self.append_diff(message, expected, actual)
        return message unless Rex.configuration.diff_enabled?

        MultiMatcherDiff.from(expected, actual).message_with_diff(message, self)
      end

      def self.diff(actual, expected)
        diff = ""
        unless actual.nil? || expected.nil?
          if all_strings?(actual, expected)
            if any_multiline_strings?(actual, expected)
              diff = diff_as_string(coerce_to_string(actual), coerce_to_string(expected))
            end
          elsif no_procs?(actual, expected) && no_numbers?(actual, expected)
            diff = diff_as_object(actual, expected)
          end
        end
        diff.to_s
      end

      # The hunks of `expected` turned into `actual`, a hunk whose context
      # overlaps the one before it written as one.
      def self.diff_as_string(actual, expected)
        return "" if over_line_limit?(actual, expected)

        output = +"\n"
        hunks = build_hunks(actual, expected)
        hunks.each_cons(2) do |previous_hunk, current_hunk|
          if current_hunk.overlaps?(previous_hunk)
            current_hunk.merge(previous_hunk)
          else
            output << previous_hunk.unified_diff
          end
          output << "\n"
        end
        output << hunks.last.unified_diff << "\n" if hunks.last
        color_diff(output)
      end

      def self.over_line_limit?(actual, expected)
        limit = Rex.configuration.diff_line_limit
        limit && [actual.split("\n").size, expected.split("\n").size].max > limit
      end

      def self.diff_as_object(actual, expected)
        diff_as_string(object_to_string(actual), object_to_string(expected))
      end

      def self.build_hunks(actual, expected)
        expected_lines = expected.split("\n").map(&:chomp)
        actual_lines = actual.split("\n").map(&:chomp)
        file_length_difference = 0
        Rex::Diff.diff(expected_lines, actual_lines).map do |piece|
          Rex::Diff::Hunk.new(expected_lines, actual_lines, piece, CONTEXT_LINES, file_length_difference).tap do |hunk|
            file_length_difference = hunk.file_length_difference
          end
        end
      end

      def self.safely_flatten(array)
        array = array.flatten(1) until array == array.flatten(1)
        array
      end

      def self.all_strings?(*objects)
        safely_flatten(objects).all? { |object| String === object }
      end

      def self.any_multiline_strings?(*objects)
        all_strings?(*objects) && safely_flatten(objects).any? { |object| object.include?("\n") }
      end

      def self.no_procs?(*objects)
        safely_flatten(objects).none? { |object| Proc === object }
      end

      def self.no_numbers?(*objects)
        safely_flatten(objects).none? { |object| Numeric === object }
      end

      # An Array of Strings is diffed as its lines, each with its own
      # newlines written as `\n`.
      def self.coerce_to_string(string_or_array)
        return string_or_array unless Array === string_or_array

        string_or_array.map do |entry|
          Array === entry ? entry.inspect : entry.to_s.gsub("\n", "\\n").gsub("\r", "\\r")
        end.join("\n")
      end

      def self.object_to_string(object)
        object = Composable.surface_descriptions_in(object)
        case object
        when Hash
          prepared = ObjectFormatter.prepare(object)
          prepared.keys.sort_by(&:to_s).map do |key|
            "#{PP.singleline_pp(key, +'')} => #{PP.singleline_pp(prepared[key], +'')},"
          end.join("\n")
        when Array
          PP.pp(ObjectFormatter.prepare(object), +"")
        when String
          object =~ /\n/ ? object : object.inspect
        else
          PP.pp(object, +"")
        end
      end

      # Each line in color: additions green, removals red, hunk headers
      # blue, and the rest in the default color.
      def self.color_diff(diff)
        return diff unless Rex.configuration.color_enabled?

        diff.lines.map do |line|
          case line[0]
          when "+" then color(line, 32)
          when "-" then color(line, 31)
          when "@" then line[1] == "@" ? color(line, 34) : color(line, 0)
          else color(line, 0)
          end
        end.join
      end

      def self.color(text, code)
        "\e[#{code}m#{text}\e[0m"
      end
    end
  end
end

module Rex
  module Matchers
    # The diffs a failure message ends with: one for a plain matcher, or one
    # per failing diffable matcher of a compound one, each with its label.
    class MultiMatcherDiff
      DEFAULT_DIFF_LABEL = "Diff:".freeze
      DESCRIPTION_MAX_LENGTH = 65

      def initialize(expected_list)
        @expected_list = expected_list
      end

      def self.from(expected, actual)
        return expected if self === expected

        new([[expected, DEFAULT_DIFF_LABEL, actual]])
      end

      def self.for_many_matchers(matchers)
        new(matchers.map { |matcher| [matcher.expected, diff_label_for(matcher), matcher.actual] })
      end

      def message_with_diff(message, differ)
        diff = diffs(differ)
        message = "#{message}\n#{diff}" unless diff.empty?
        message
      end

      class << self
        private

        def diff_label_for(matcher)
          "Diff for (#{truncated(ObjectFormatter.format(matcher))}):"
        end

        def truncated(description)
          return description if description.length <= DESCRIPTION_MAX_LENGTH

          description[0...(DESCRIPTION_MAX_LENGTH - 3)] << "..."
        end
      end

      private

      def diffs(differ)
        @expected_list.map do |(expected, diff_label, actual)|
          diff = differ.diff(actual, expected)
          next if diff.strip.empty?

          if diff == "\e[0m\n\e[0m"
            "#{diff_label}\n" \
              "  <The diff is empty, are your objects producing identical `#inspect` output?>"
          else
            "#{diff_label}#{diff}"
          end
        end.compact.join("\n")
      end
    end
  end
end
