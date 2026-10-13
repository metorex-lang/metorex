module Rex
  module Matchers
    # The source text of a block handed to a matcher, read from the file the
    # block was written in, so a message can quote `list.size` rather than say
    # "result". Answers nil when the line holds no single such block.
    module BlockSnippet
      OPENERS = { "{" => "}", "(" => ")", "[" => "]" }.freeze

      def self.for(block, method_name)
        file, line = block.source_location
        return nil unless file && line && File.exist?(file)

        source = File.readlines(file).drop(line - 1).join
        starts = source.lines.first.to_s.enum_for(:scan, /\b#{Regexp.escape(method_name)}\s*(?:\([^()]*\))?\s*(\{|do\b)/).map { Regexp.last_match }
        return nil unless starts.size == 1

        match = starts.first
        opener = match[1]
        body_start = match.end(0)
        body = opener == "{" ? brace_body(source, body_start) : do_body(source, body_start)
        return nil unless body

        body = body.sub(/\A\s*\|[^|]*\|/, "")
        lines = body.strip.lines.map(&:rstrip)
        return nil if lines.empty?

        lines.size == 1 ? lines.first.strip : lines.map(&:strip).join("\n")
      rescue StandardError
        nil
      end

      def self.brace_body(source, from)
        depth = 1
        index = from
        quote = nil
        while index < source.length
          character = source[index]
          if quote
            if character == "\\"
              index += 1
            elsif character == quote
              quote = nil
            end
          elsif character == '"' || character == "'"
            quote = character
          elsif character == "{"
            depth += 1
          elsif character == "}"
            depth -= 1
            return source[from...index] if depth.zero?
          end
          index += 1
        end
        nil
      end

      def self.do_body(source, from)
        depth = 1
        source[from..].scan(/\b(do|end)\b|./m) do
          word = Regexp.last_match(1)
          next unless word

          depth += word == "do" ? 1 : -1
          return source[from...(from + Regexp.last_match.begin(0))] if depth.zero?
        end
        nil
      end
    end
  end
end
