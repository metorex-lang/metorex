module Rex
  module Core
    module Formatters
      class SyntaxHighlighter
        def initialize(configuration)
          @configuration = configuration
        end

        def highlight(lines)
          implementation.highlight_syntax(lines)
        end

        def self.attempt_to_add_rspec_terms_to_coderay_keywords
          CodeRay::Scanners::Ruby::Patterns::IDENT_KIND.add(%w[
            describe context
            it specify
            before after around
            let subject
            expect allow
          ], :keyword)
        rescue Exception
        end

      private

        if Rex::Support::OS.windows?
          def implementation
            WindowsImplementation
          end
        else
          def implementation
            return color_enabled_implementation if @configuration.color_enabled?
            NoSyntaxHighlightingImplementation
          end
        end

        def color_enabled_implementation
          @color_enabled_implementation ||= begin
            require 'coderay'
            self.class.attempt_to_add_rspec_terms_to_coderay_keywords
            CodeRayImplementation
          rescue LoadError
            NoSyntaxHighlightingImplementation
          end
        end

        module CodeRayImplementation
          RESET_CODE = "\e[0m"

          def self.highlight_syntax(lines)
            highlighted = begin
              CodeRay.encode(lines.join("\n"), :ruby, :terminal)
            rescue Support::AllExceptionsExceptOnesWeMustNotRescue
              return lines
            end

            highlighted.split("\n").map do |line|
              line.sub(/\S/) { |char| char.insert(0, RESET_CODE) }
            end
          end
        end

        module NoSyntaxHighlightingImplementation
          def self.highlight_syntax(lines)
            lines
          end
        end

        WindowsImplementation = NoSyntaxHighlightingImplementation
      end
    end
  end
end
