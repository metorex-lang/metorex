
module Rex
  module Core
    module Warnings
      def deprecate(deprecated, data={})
        Rex.configuration.reporter.deprecation(
          {
            :deprecated => deprecated,
            :call_site => CallerFilter.first_non_rspec_line
          }.merge(data)
        )
      end

      def warn_deprecation(message, opts={})
        Rex.configuration.reporter.deprecation opts.merge(:message => message)
      end

      def warn_with(message, options={})
        if options[:use_spec_location_as_call_site]
          message += "." unless message.end_with?(".")

          if Rex.current_example
            message += " Warning generated from spec at `#{Rex.current_example.location}`."
          end
        end

        super(message, options)
      end
    end
  end
end
