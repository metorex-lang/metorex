module Rex
  module Core
    module Formatters
      module ConsoleCodes
        VT100_CODES =
          {
            :black         => 30,
            :red           => 31,
            :green         => 32,
            :yellow        => 33,
            :blue          => 34,
            :magenta       => 35,
            :cyan          => 36,
            :white         => 37,
            :bold_black   => '1;30',
            :bold_red     => '1;31',
            :bold_green   => '1;32',
            :bold_yellow  => '1;33',
            :bold_blue    => '1;34',
            :bold_magenta => '1;35',
            :bold_cyan    => '1;36',
            :bold_white   => '1;37',
            :bold          => 1,
          }
        VT100_CODE_VALUES = VT100_CODES.invert

        module_function

        def config_colors_to_methods
          @config_colors_to_methods ||=
            Configuration.instance_methods.grep(/_color\z/).inject({}) do |hash, method|
              hash[method.to_s.sub(/_color\z/, '').to_sym] = method
              hash
            end
        end

        def console_code_for(code_or_symbol)
          if (config_method = config_colors_to_methods[code_or_symbol])
            console_code_for Rex.configuration.__send__(config_method)
          elsif VT100_CODE_VALUES.key?(code_or_symbol)
            code_or_symbol
          else
            VT100_CODES.fetch(code_or_symbol) do
              console_code_for(:white)
            end
          end
        end

        def wrap(text, code_or_symbol)
          if Rex.configuration.color_enabled?
            "\e[#{console_code_for(code_or_symbol)}m#{text}\e[0m"
          else
            text
          end
        end
      end
    end
  end
end
