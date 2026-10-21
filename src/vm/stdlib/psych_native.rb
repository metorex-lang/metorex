# What psych.so defines under MRI, written over Metorex's own YAML reader
# and writer in place of libyaml.

require "psych/syntax_error"

module Psych
  # The libyaml release whose behavior the reader and writer follow.
  def self.libyaml_version
    [0, 2, 5]
  end

  class Parser
    ANY = 0
    UTF8 = 1
    UTF16LE = 2
    UTF16BE = 3

    READ_LENGTH = 16_384
    private_constant :READ_LENGTH

    def mark
      @mark || Mark.new(0, 0, 0)
    end

    private

    def _native_parse(handler, yaml, path)
      events, failure = Psych.__parse__(__yaml_text__(yaml))
      internal = Encoding.default_internal
      events.each do |event|
        start_line, start_column, end_line, end_column, index, line, column, name, *fields = event
        fields = __transcoded__(fields, internal) if internal
        @mark = Mark.new(index, line, column)
        handler.event_location(start_line, start_column, end_line, end_column)
        handler.public_send(name, *fields)
      end
      return self if failure.nil?
      problem, context, line, column, offset = failure
      raise Psych::SyntaxError.new(path, line, column, offset, problem, context)
    end

    # The text a parse reads, as the UTF-8 bytes libyaml reads it in.
    def __yaml_text__(yaml)
      if yaml.respond_to?(:read)
        # libyaml reads a stream a buffer at a time, through `read(length)`.
        text = "".b
        while (chunk = yaml.read(READ_LENGTH))
          piece = String.try_convert(chunk)
          raise TypeError, "no implicit conversion of #{chunk.class} into String" if piece.nil?
          text << piece.b
          break if piece.empty?
        end
        encoding = yaml.external_encoding if yaml.respond_to?(:external_encoding)
        # A stream in an encoding libyaml does not read is handed over as
        # its bytes, as one in no encoding at all is.
        case encoding
        when Encoding::UTF_8, Encoding::US_ASCII then text.force_encoding(Encoding::UTF_8)
        when Encoding::UTF_16LE, Encoding::UTF_16BE then text.force_encoding(encoding)
        end
      else
        text = String.try_convert(yaml)
        raise TypeError, "no implicit conversion of #{yaml.class} into String" if text.nil?
      end
      __as_utf8__(text)
    end

    # The strings an event carries, in the encoding the program asked
    # strings it reads to arrive in.
    def __transcoded__(fields, internal)
      fields.map do |field|
        case field
        when String
          begin
            field.encode(internal)
          rescue EncodingError
            field
          end
        when Array then __transcoded__(field, internal)
        else field
        end
      end
    end

    def __as_utf8__(text)
      case text.encoding
      when Encoding::UTF_8, Encoding::US_ASCII
        text
      when Encoding::UTF_16LE, Encoding::UTF_16BE
        text.encode(Encoding::UTF_8)
      when Encoding::BINARY
        bytes = text.byteslice(0, 2).bytes
        return text.dup.force_encoding(Encoding::UTF_16BE).encode(Encoding::UTF_8) if bytes == [0xFE, 0xFF]
        return text.dup.force_encoding(Encoding::UTF_16LE).encode(Encoding::UTF_8) if bytes == [0xFF, 0xFE]
        text
      else
        begin
          text.encode(Encoding::UTF_8)
        rescue EncodingError
          text
        end
      end
    end
  end

  class Handler
  end

  class Emitter < Handler
    def initialize(io, options = nil)
      @io = io
      if options
        @line_width = options.line_width
        @indentation = options.indentation
        @canonical = options.canonical == true
      else
        @line_width = 0
        @indentation = 2
        @canonical = false
      end
      @emitter = Psych.__emitter__(:new, @line_width, @indentation, @canonical)
    end

    def start_stream(encoding)
      raise TypeError, "wrong argument type #{encoding.class} (expected Integer)" unless encoding.is_a?(Integer)
      __emit__(:start_stream)
    end

    def end_stream
      __emit__(:end_stream)
    end

    def start_document(version, tag_directives, implicit)
      __check_type__(version, Array)
      version = version.empty? ? [] : [Integer(version[0]), Integer(version[1])]
      tags = []
      if tag_directives
        __check_type__(tag_directives, Array)
        tag_directives.each do |pair|
          __check_type__(pair, Array)
          raise RuntimeError, "tag tuple must be of length 2" if pair.size < 2
          tags << [__text__(pair[0], true), __text__(pair[1], true)]
        end
      end
      __emit__(:start_document, version, tags, implicit ? true : false)
    end

    def end_document(implicit = true)
      __emit__(:end_document, implicit ? true : false)
    end

    def scalar(value, anchor, tag, plain, quoted, style)
      __emit__(:scalar, __text__(value, true), __text__(anchor), __text__(tag),
               plain ? true : false, quoted ? true : false, Integer(style))
    end

    def start_sequence(anchor, tag, implicit, style)
      __emit__(:start_sequence, __text__(anchor), __text__(tag), implicit ? true : false, Integer(style))
    end

    def end_sequence
      __emit__(:end_sequence)
    end

    def start_mapping(anchor, tag, implicit, style)
      __emit__(:start_mapping, __text__(anchor), __text__(tag), implicit ? true : false, Integer(style))
    end

    def end_mapping
      __emit__(:end_mapping)
    end

    def alias(anchor)
      __emit__(:alias, __text__(anchor))
    end

    def canonical
      @canonical
    end

    def canonical=(value)
      @canonical = value ? true : false
    end

    def indentation
      @indentation
    end

    def indentation=(value)
      @indentation = value
    end

    def line_width
      @line_width
    end

    def line_width=(value)
      @line_width = value
    end

    private

    # The TypeError Check_Type raises for a value of the wrong kind.
    def __check_type__(value, kind)
      return if value.is_a?(kind)
      shown = { nil => "nil", true => "true", false => "false" }.fetch(value) { value.class.to_s }
      raise TypeError, "wrong argument type #{shown} (expected #{kind.name})"
    end

    def __text__(value, required = false)
      return nil if value.nil? && !required
      text = String.try_convert(value)
      if text.nil?
        shown = { nil => "nil", true => "true", false => "false" }.fetch(value) { value.class.to_s }
        raise TypeError, "no implicit conversion of #{shown} into String"
      end
      # A String subclass's text is handed over as a plain String.
      text = String.new(text) unless text.instance_of?(String)
      text.encoding == Encoding::UTF_8 ? text : text.encode(Encoding::UTF_8)
    end

    def __emit__(name, *fields)
      written = Psych.__emitter__(:emit, @emitter, name, *fields)
      @io.write(written) unless written.nil? || written.empty?
      self
    end
  end

  class ClassLoader
    private

    def path2class(path)
      path.to_s.split("::").inject(Object) do |holder, name|
        next holder if name.empty?
        holder.const_get(name, false)
      rescue NameError
        raise ArgumentError, "undefined class/module #{path}"
      end
    end
  end

  module Visitors
    class Visitor
    end

    class ToRuby < Visitor
      private

      def build_exception(klass, message)
        made = klass.allocate
        made.__restore_message__(message)
        made
      end
    end

    class YAMLTree < Visitor
    end
  end
end
