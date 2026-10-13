module Rex
  module Support
    # A Ruby source file read through Ripper: its lines, tokens, and syntax
    # tree, which RSpec's block snippet extractor walks.
    class Source
      attr_reader :source, :path

      def self.from_file(path)
        new(::File.read(path), path)
      end

      def initialize(source_string, path = nil)
        @source = source_string
        @path = path ? ::File.expand_path(path) : "(string)"
      end

      def lines
        @lines ||= source.split("\n")
      end

      def inspect
        "#<#{self.class} #{path}>"
      end

      def ast
        @ast ||= begin
          require "ripper"
          sexp = Ripper.sexp(source)
          raise SyntaxError unless sexp

          Node.new(sexp)
        end
      end

      def tokens
        @tokens ||= begin
          require "ripper"
          Token.tokens_from_ripper_tokens(Ripper.lex(source))
        end
      end

      def nodes_by_line_number
        @nodes_by_line_number ||= begin
          nodes_by_line_number = ast.select(&:location).group_by { |node| node.location.line }
          Hash.new { |hash, key| hash[key] = [] }.merge(nodes_by_line_number)
        end
      end

      def tokens_by_line_number
        @tokens_by_line_number ||= begin
          tokens_by_line_number = tokens.group_by { |token| token.location.line }
          Hash.new { |hash, key| hash[key] = [] }.merge(tokens_by_line_number)
        end
      end

      Location = Struct.new(:line, :column) do
        include Comparable

        def self.location?(array)
          array.is_a?(Array) && array.size == 2 && array.all? { |element| element.is_a?(Integer) }
        end

        def <=>(other)
          line_comparison = (line <=> other.line)
          return line_comparison unless line_comparison == 0

          column <=> other.column
        end
      end

      # A node of the tree `Ripper.sexp` gives.
      class Node
        include Enumerable

        attr_reader :sexp, :parent

        def self.sexp?(array)
          array.is_a?(Array) && array.first.is_a?(Symbol)
        end

        def initialize(ripper_sexp, parent = nil)
          @sexp = ripper_sexp.freeze
          @parent = parent
        end

        def type
          sexp[0]
        end

        def args
          @args ||= raw_args.map do |raw_arg|
            if Node.sexp?(raw_arg)
              Node.new(raw_arg, self)
            elsif Location.location?(raw_arg)
              Location.new(*raw_arg)
            elsif raw_arg.is_a?(Array)
              ExpressionSequenceNode.new(raw_arg, self)
            else
              raw_arg
            end
          end.freeze
        end

        def children
          @children ||= args.select { |arg| arg.is_a?(Node) }.freeze
        end

        def location
          @location ||= args.find { |arg| arg.is_a?(Location) }
        end

        # Visits the node and its descendants, breadth first.
        def each
          return to_enum(__method__) unless block_given?

          node_queue = [self]
          while (current_node = node_queue.shift)
            yield current_node
            node_queue.concat(current_node.children)
          end
        end

        def each_ancestor
          return to_enum(__method__) unless block_given?

          current_node = self
          while (current_node = current_node.parent)
            yield current_node
          end
        end

        def inspect
          "#<#{self.class} #{type}>"
        end

        private

        def raw_args
          sexp[1..] || []
        end
      end

      # A list of expressions, which Ripper writes as an Array that names no
      # node type.
      class ExpressionSequenceNode < Node
        def type
          :_expression_sequence
        end

        private

        def raw_args
          sexp
        end
      end

      # A token `Ripper.lex` gives.
      class Token
        CLOSING_TYPES_BY_OPENING_TYPE = {
          on_lbracket: :on_rbracket,
          on_lparen: :on_rparen,
          on_lbrace: :on_rbrace,
          on_heredoc_beg: :on_heredoc_end
        }.freeze

        CLOSING_KEYWORDS_BY_OPENING_KEYWORD = {
          "def" => "end",
          "do" => "end"
        }.freeze

        attr_reader :token

        def self.tokens_from_ripper_tokens(ripper_tokens)
          ripper_tokens.map { |ripper_token| new(ripper_token) }.freeze
        end

        def initialize(ripper_token)
          @token = ripper_token.freeze
        end

        def location
          @location ||= Location.new(*token[0])
        end

        def type
          token[1]
        end

        def string
          token[2]
        end

        def ==(other)
          token == other.token
        end
        alias eql? ==

        def inspect
          "#<#{self.class} #{type} #{string.inspect}>"
        end

        def keyword?
          type == :on_kw
        end

        def equals_operator?
          type == :on_op && string == "="
        end

        def opening?
          opening_delimiter? || opening_keyword?
        end

        def closed_by?(other)
          delimiter_closed_by?(other) || keyword_closed_by?(other)
        end

        private

        def opening_delimiter?
          CLOSING_TYPES_BY_OPENING_TYPE.key?(type)
        end

        def opening_keyword?
          return false unless keyword?

          CLOSING_KEYWORDS_BY_OPENING_KEYWORD.key?(string)
        end

        def delimiter_closed_by?(other)
          other.type == CLOSING_TYPES_BY_OPENING_TYPE[type]
        end

        def keyword_closed_by?(other)
          return false unless keyword?
          return true if other.string == CLOSING_KEYWORDS_BY_OPENING_KEYWORD[string]

          # An endless method, `def name = value`, is closed by its `=`.
          string == "def" && other.equals_operator? && location.line == other.location.line
        end
      end
    end
  end
end
