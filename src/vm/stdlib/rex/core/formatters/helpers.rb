require "rex/core/shell_escape"

module Rex
  module Core
    module Formatters
      module Helpers
        SUB_SECOND_PRECISION = 5

        DEFAULT_PRECISION = 2

        def self.format_duration(duration)
          precision = case
                      when duration < 1 then    SUB_SECOND_PRECISION
                      when duration < 120 then  DEFAULT_PRECISION
                      when duration < 300 then  1
                      else                  0
                      end

          if duration > 60
            minutes = (duration.round / 60).to_i
            seconds = (duration - minutes * 60)

            "#{pluralize(minutes, 'minute')} #{pluralize(format_seconds(seconds, precision), 'second')}"
          else
            pluralize(format_seconds(duration, precision), 'second')
          end
        end

        def self.format_seconds(float, precision=nil)
          return '0' if float < 0
          precision ||= (float < 1) ? SUB_SECOND_PRECISION : DEFAULT_PRECISION
          formatted = "%.#{precision}f" % float
          strip_trailing_zeroes(formatted)
        end

        def self.strip_trailing_zeroes(string)
          string.sub(/(?:(\..*[^0])0+|\.0+)$/, '\1')
        end
        private_class_method :strip_trailing_zeroes

        def self.pluralize(count, string)
          pluralized_string = if count.to_f == 1
                                string
                              elsif string.end_with?('s')
                                "#{string}es"
                              else
                                "#{string}s"
                              end

          "#{count} #{pluralized_string}"
        end

        def self.organize_ids(ids)
          grouped = ids.inject(Hash.new { |h, k| h[k] = [] }) do |hash, id|
            file, id = Example.parse_id(id)
            hash[file] << id
            hash
          end

          grouped.sort_by(&:first).map do |file, grouped_ids|
            grouped_ids = grouped_ids.sort_by { |id| id.split(':').map(&:to_i) }
            id = Metadata.id_from(:rerun_file_path => file, :scoped_id => grouped_ids.join(','))
            ShellEscape.conditionally_quote(id)
          end
        end
      end
    end
  end
end
