require("prism")
require("digest")

source = <<~'RUBY'
  module Billing
    RATE = 0x1F
    LIMIT = 123456789012345678901234567890
    class Invoice < Base::Record
      attr_reader :total

      def initialize(items, discount = 0.25, *rest, tax:, note: nil, **options, &block)
        @items, @discount = items, discount
        @total = items.sum { |item| item[:price] * 2r + 3i }
        $last = "invoice #{items.size} for #{@total}"
      end

      def self.parse(text) = text.scan(/\d+/i).map(&:to_i)

      def status
        case @total
        in Integer => amount if amount > 100 then :large
        in [first, *] | {name: String} then :shaped
        else :small
        end
      rescue ArgumentError, TypeError => error
        warn(error.message)
        retry
      ensure
        @checked ||= true
      end
    end
  end

  values = [1, -2, 3.5, nil, true, :rate, %w[a b], 1..5, ->(x) { x * 2 }]
  values.each_with_index { puts(it) }
  values.each_slice(2) { warn(_1, _2) }
  note = <<~TEXT
    shipping #{values.size}
  TEXT
  defined?(@total) && !note.empty? || BEGIN_NOT
  __END__
  unparsed
RUBY

result = Prism.parse(source)
tree = result.value.inspect
p(tree.lines.size)
p(Digest::SHA256.hexdigest(tree))
p(result.value.statements.body.map(&:type))
p(result.data_loc&.slice)
p(result.comments.size)

lexed = Prism.parse_lex("rate = [1, 'two', :three] # note\n")
lexed.value.last.each { |token, state| p([token.type, token.value, token.location.start_offset, state]) }
p(Digest::SHA256.hexdigest(lexed.value.first.inspect))

frozen = Prism.parse("total = items.sum(0)", freeze: true).value
p(frozen.frozen?)
p(frozen.statements.body.first.frozen?)
p(frozen.statements.body.first.name_loc.frozen?)
p(frozen.statements.body.first.value.arguments.arguments.frozen?)

p(Prism.parse("def go(a) = a").value.statements.body.first.parameters.requireds.map(&:name))
p(Prism.parse("x = \"\\u00e9t\\xC3\\xA9\"").value.statements.body.first.value.unescaped)
p(Prism.lex("a\n  b").value.map { |token, _state| token.location.start_column })
