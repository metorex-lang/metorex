module Rex
  # The source a failure quotes, read through Metorex's lexer:
  # `__statement_lines__(path, line)` answers the first and last line of the
  # statement `line` falls in, and `__expect_argument__(path, line)` the text
  # of the value `expect(...)` is handed on `line`.
  module Source
  end

  module Core
    module Formatters
      # The lines of the statement a failure's backtrace line falls in, up
      # to `max_line_count` of them.
      module SnippetExtractor
        NoSuchFileError = Class.new(StandardError)
        NoSuchLineError = Class.new(StandardError)

        def self.extract_line_at(file_path, line_number)
          line = lines_of(file_path)[line_number - 1]
          raise NoSuchLineError unless line

          line
        end

        def self.extract_expression_lines_at(file_path, beginning_line_number, max_line_count = nil)
          return [extract_line_at(file_path, beginning_line_number)] if max_line_count == 1

          first, last = Rex::Source.__statement_lines__(File.expand_path(file_path), beginning_line_number)
          return [extract_line_at(file_path, beginning_line_number)] unless first

          last = [last, first + max_line_count - 1].min if max_line_count
          lines_of(file_path)[(first - 1)..(last - 1)]
        end

        def self.least_indentation_from(lines)
          lines.map { |line| line[/^[ \t]*/] }.min
        end

        def self.lines_of(file_path)
          raise NoSuchFileError unless File.exist?(file_path)

          Rex.world.source_lines_for(file_path)
        end
      end
    end

    class World
      def source_lines_for(path)
        (@source_lines_by_path ||= {})[path] ||= File.read(path).split("\n")
      end
    end
  end
end
