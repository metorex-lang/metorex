module Rex
  module Core
    module ShellEscape
      module_function

      def quote(argument)
        "'#{argument.to_s.gsub("'", "\\\\'")}'"
      end

      if Rex::Support::OS.windows?
        alias escape quote
      else
        require 'shellwords'

        def escape(shell_command)
          Shellwords.escape(shell_command.to_s)
        end
      end

      SHELLS_ALLOWING_UNQUOTED_IDS = %w[ bash ksh fish ]

      def conditionally_quote(id)
        return id if shell_allows_unquoted_ids?
        quote(id)
      end

      def shell_allows_unquoted_ids?
        SHELLS_ALLOWING_UNQUOTED_IDS.include?(ENV['SHELL'].to_s.split('/').last)
      end
    end
  end
end
