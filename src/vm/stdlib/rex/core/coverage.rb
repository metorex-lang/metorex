module Rex
  module Core
    # `metorex spec --coverage`: which lines and branches of the files the
    # suite loaded ran, reported as a summary, an LCOV file, or a JSON file.
    module CoverageReport
      FORMATS = %w[summary lcov json].freeze

      def self.start
        require "coverage"
        ::Coverage.start(lines: true)
      end

      # The counts so far, as plain data a worker can send to the parent:
      # each file's line counts and branch sites.
      def self.collect
        branches = ::Coverage.__coverage_branches__.to_h
        lines = ::Coverage.result.transform_values { |value| value.is_a?(Hash) ? value[:lines] : value }
        { lines: lines, branches: branches }
      end

      # Adds the counts of `other` to those of `into`, file by file.
      def self.merge(into, other)
        other[:lines].each do |path, counts|
          held = into[:lines][path]
          into[:lines][path] = held ? held.zip(counts).map { |mine, theirs| mine.nil? ? theirs : mine + theirs.to_i } : counts
        end
        other[:branches].each do |path, sites|
          held = into[:branches][path]
          into[:branches][path] = if held
                                    held.zip(sites).map do |(kind, line, arms), (_, _, their_arms)|
                                      [kind, line, arms.zip(their_arms).map { |(arm_kind, arm_line, count), theirs| [arm_kind, arm_line, count + theirs[2]] }]
                                    end
                                  else
                                    sites
                                  end
        end
        into
      end

      FileResult = Struct.new(:path, :lines, :sites) do
        def relevant_lines
          lines.compact.size
        end

        def covered_lines
          lines.count { |count| count && count.positive? }
        end

        def missed_line_numbers
          lines.each_with_index.select { |count, _| count&.zero? }.map { |_, index| index + 1 }
        end

        def arms
          sites.flat_map { |_, _, site_arms| site_arms }
        end

        def covered_arms
          arms.count { |_, _, count| count.positive? }
        end

        def line_percent
          CoverageReport.percent(covered_lines, relevant_lines)
        end

        def branch_percent
          CoverageReport.percent(covered_arms, arms.size)
        end
      end

      def self.percent(part, whole)
        whole.zero? ? 100.0 : part * 100.0 / whole
      end

      # The files that count: everything loaded except the spec files and
      # what sits under the spec directory, and except what an exclusion
      # names.
      def self.files(data, configuration)
        spec_directory = File.expand_path(configuration.default_path) + "/"
        loaded_specs = configuration.loaded_spec_files
        data[:lines].keys.sort.filter_map do |path|
          absolute = File.expand_path(path)
          next if loaded_specs.include?(absolute) || absolute.start_with?(spec_directory)

          relative = Rex::Core.relative_path(absolute).sub(%r{\A\./}, "")
          next if configuration.coverage_exclude.any? { |pattern| excluded?(pattern, relative) }

          FileResult.new(relative, data[:lines][path], data[:branches][path] || [])
        end
      end

      def self.excluded?(pattern, relative)
        pattern.is_a?(Regexp) ? pattern.match?(relative) : File.fnmatch?(pattern, relative, File::FNM_PATHNAME | File::FNM_EXTGLOB)
      end

      # Writes each format asked for and answers whether the line coverage
      # meets the threshold.
      def self.report(data, configuration)
        results = files(data, configuration)
        relevant = results.sum(&:relevant_lines)
        covered = results.sum(&:covered_lines)
        arms = results.sum { |result| result.arms.size }
        covered_arms = results.sum(&:covered_arms)
        line_percent = percent(covered, relevant)
        branch_percent = percent(covered_arms, arms)

        configuration.coverage_formats.each do |format|
          case format
          when "summary"
            write_summary(configuration.output_stream, results, covered, relevant, covered_arms, arms)
          when "lcov"
            write_file(configuration, "lcov.info", lcov(results))
          when "json"
            require "json"
            write_file(configuration, "coverage.json", JSON.pretty_generate(json(results, line_percent, branch_percent)) + "\n")
          end
        end

        threshold = configuration.coverage_threshold
        return true unless threshold && line_percent < threshold

        shown = threshold == threshold.to_i ? threshold.to_i : threshold
        configuration.error_stream.puts "\nLine coverage, #{format('%.2f', line_percent)}%, is below the threshold of #{shown}%."
        false
      end

      def self.write_summary(output, results, covered, relevant, covered_arms, arms)
        output.puts
        output.puts "Coverage: #{format('%.2f', percent(covered, relevant))}% of lines (#{covered} of #{relevant}) " \
                    "and #{format('%.2f', percent(covered_arms, arms))}% of branches (#{covered_arms} of #{arms}) " \
                    "in #{Formatters.pluralize(results.size, 'file')}"
        width = results.map { |result| result.path.size }.max || 0
        results.each do |result|
          line = "  #{result.path.ljust(width)}  #{format('%6.2f', result.line_percent)}% of lines  " \
                 "#{format('%6.2f', result.branch_percent)}% of branches"
          missed = result.missed_line_numbers
          line += "  not run: #{missed.join(', ')}" unless missed.empty?
          output.puts line
        end
      end

      def self.write_file(configuration, name, content)
        directory = File.expand_path(configuration.coverage_dir)
        FileUtils.mkdir_p(directory)
        File.write(File.join(directory, name), content)
      end

      def self.lcov(results)
        results.map do |result|
          lines = ["TN:", "SF:#{File.expand_path(result.path)}"]
          result.lines.each_with_index do |count, index|
            lines << "DA:#{index + 1},#{count}" unless count.nil?
          end
          lines << "LF:#{result.relevant_lines}"
          lines << "LH:#{result.covered_lines}"
          result.sites.each_with_index do |(_, line, arms), block|
            reached = arms.any? { |_, _, count| count.positive? }
            arms.each_with_index do |(_, _, count), branch|
              lines << "BRDA:#{line},#{block},#{branch},#{reached ? count : '-'}"
            end
          end
          lines << "BRF:#{result.arms.size}"
          lines << "BRH:#{result.covered_arms}"
          lines << "end_of_record"
          lines.join("\n")
        end.join("\n") + "\n"
      end

      def self.json(results, line_percent, branch_percent)
        {
          line_percent: line_percent.round(2),
          branch_percent: branch_percent.round(2),
          files: results.to_h do |result|
            [result.path, {
              line_percent: result.line_percent.round(2),
              branch_percent: result.branch_percent.round(2),
              lines: result.lines,
              branches: result.sites.map do |kind, line, arms|
                { kind: kind, line: line, arms: arms.map { |arm_kind, arm_line, count| { kind: arm_kind, line: arm_line, count: count } } }
              end
            }]
          end
        }
      end
    end
  end
end
