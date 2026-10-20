require "rspec/support/directory_maker"

module Rex
  module Core
    class ExampleStatusPersister
      def self.load_from(file_name)
        return [] unless File.exist?(file_name)
        ExampleStatusParser.parse(File.read(file_name))
      end

      def self.persist(examples, file_name)
        new(examples, file_name).persist
      end

      def initialize(examples, file_name)
        @examples  = examples
        @file_name = file_name
      end

      def persist
        Rex::Support::DirectoryMaker.mkdir_p(File.dirname(@file_name))
        File.open(@file_name, File::RDWR | File::CREAT) do |f|
          f.flock(File::LOCK_EX)
          unparsed_previous_runs = f.read
          f.rewind
          f.write(dump_statuses(unparsed_previous_runs))
          f.flush
          f.truncate(f.pos)
        end
      end

    private

      def dump_statuses(unparsed_previous_runs)
        statuses_from_previous_runs = ExampleStatusParser.parse(unparsed_previous_runs)
        merged_statuses = ExampleStatusMerger.merge(statuses_from_this_run, statuses_from_previous_runs)
        ExampleStatusDumper.dump(merged_statuses)
      end

      def statuses_from_this_run
        @examples.map do |ex|
          result = ex.execution_result

          {
            :example_id => ex.id,
            :status     => result.status ? result.status.to_s : Configuration::UNKNOWN_STATUS,
            :run_time   => result.run_time ? Formatters::Helpers.format_duration(result.run_time) : ""
          }
        end
      end
    end

    class ExampleStatusMerger
      def self.merge(this_run, from_previous_runs)
        new(this_run, from_previous_runs).merge
      end

      def initialize(this_run, from_previous_runs)
        @this_run           = hash_from(this_run)
        @from_previous_runs = hash_from(from_previous_runs)
        @file_exists_cache  = Hash.new { |hash, file| hash[file] = File.exist?(file) }
      end

      def merge
        delete_previous_examples_that_no_longer_exist

        @this_run.merge(@from_previous_runs) do |_ex_id, new, old|
          new.fetch(:status) == Configuration::UNKNOWN_STATUS ? old : new
        end.values.sort_by(&method(:sort_value_from))
      end

    private

      def hash_from(example_list)
        example_list.inject({}) do |hash, example|
          hash[example.fetch(:example_id)] = example
          hash
        end
      end

      def delete_previous_examples_that_no_longer_exist
        @from_previous_runs.delete_if do |ex_id, _|
          example_must_no_longer_exist?(ex_id)
        end
      end

      def example_must_no_longer_exist?(ex_id)
        return false if @this_run.key?(ex_id)

        spec_file = spec_file_from(ex_id)

        return true if loaded_spec_files.include?(spec_file)

        !@file_exists_cache[spec_file]
      end

      def loaded_spec_files
        @loaded_spec_files ||= Set.new(@this_run.keys.map(&method(:spec_file_from)))
      end

      def spec_file_from(ex_id)
        ex_id.split("[").first
      end

      def sort_value_from(example)
        file, scoped_id = Example.parse_id(example.fetch(:example_id))
        [file, *scoped_id.split(":").map(&method(:Integer))]
      end
    end

    class ExampleStatusDumper
      def self.dump(examples)
        new(examples).dump
      end

      def initialize(examples)
        @examples = examples
      end

      def dump
        return nil if @examples.empty?
        (formatted_header_rows + formatted_value_rows).join("\n") << "\n"
      end

    private

      def formatted_header_rows
        @formatted_header_rows ||= begin
          dividers = column_widths.map { |w| "-" * w }
          [formatted_row_from(headers.map(&:to_s)), formatted_row_from(dividers)]
        end
      end

      def formatted_value_rows
        @formatted_value_rows ||= rows.map do |row|
          formatted_row_from(row)
        end
      end

      def rows
        @rows ||= @examples.map { |ex| ex.values_at(*headers) }
      end

      def formatted_row_from(row_values)
        padded_values = row_values.each_with_index.map do |value, index|
          value.ljust(column_widths[index])
        end

        padded_values.join(" | ") << " |"
      end

      def headers
        @headers ||= @examples.first.keys
      end

      def column_widths
        @column_widths ||= begin
          value_sets = rows.transpose

          headers.each_with_index.map do |header, index|
            values = value_sets[index] << header.to_s
            values.map(&:length).max
          end
        end
      end
    end

    class ExampleStatusParser
      def self.parse(string)
        new(string).parse
      end

      def initialize(string)
        @header_line, _, *@row_lines = string.lines.to_a
      end

      def parse
        @row_lines.map { |line| parse_row(line) }
      end

    private

      def parse_row(line)
        Hash[headers.zip(split_line(line))]
      end

      def headers
        @headers ||= split_line(@header_line).grep(/\S/).map(&:to_sym)
      end

      def split_line(line)
        line.split(/\s+\|\s+?/, -1)
      end
    end
  end
end
