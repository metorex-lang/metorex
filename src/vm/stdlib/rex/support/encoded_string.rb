# frozen_string_literal: true

module Rex
  module Support
    class EncodedString
      UTF_8    = "UTF-8"
      US_ASCII = "US-ASCII"

      REPLACE = "?"

      def initialize(string, encoding=nil)
        @encoding = encoding
        @source_encoding = detect_source_encoding(string)
        @string = matching_encoding(string)
      end
      attr_reader :source_encoding

      delegated_methods = String.instance_methods.map(&:to_s) & %w[eql? lines == encoding empty?]
      delegated_methods.each do |name|
        define_method(name) { |*args, &block| @string.__send__(name, *args, &block) }
      end

      def <<(string)
        @string << matching_encoding(string)
      end

      if Ruby.jruby?
        def split(regex_or_string)
          @string.split(matching_encoding(regex_or_string))
        rescue ArgumentError
          remove_invalid_bytes(@string).split regex_or_string
        end
      else
        def split(regex_or_string)
          @string.split(matching_encoding(regex_or_string))
        end
      end

      def to_s
        @string
      end
      alias :to_str :to_s

      if String.method_defined?(:encoding)

        private

        def matching_encoding(string)
          string = remove_invalid_bytes(string)
          string.encode(@encoding)
        rescue Encoding::UndefinedConversionError, Encoding::InvalidByteSequenceError
          string.encode(@encoding, :invalid => :replace, :undef => :replace, :replace => REPLACE)
        rescue Encoding::ConverterNotFoundError
          string.dup.force_encoding(@encoding).encode(:invalid => :replace, :replace => REPLACE)
        end

        if String.method_defined?(:scrub)
          def remove_invalid_bytes(string)
            string.scrub(REPLACE)
          end
        else
          def remove_invalid_bytes(string)
            string.chars.map do |char|
              char.valid_encoding? ? char : REPLACE
            end.join
          end
        end

        def detect_source_encoding(string)
          string.encoding
        end

        def self.pick_encoding(source_a, source_b)
          Encoding.compatible?(source_a, source_b) || Encoding.default_external
        end
      else

        def self.pick_encoding(_source_a, _source_b)
        end

        private

        def matching_encoding(string)
          string
        end

        def detect_source_encoding(_string)
          US_ASCII
        end
      end
    end
  end
end
