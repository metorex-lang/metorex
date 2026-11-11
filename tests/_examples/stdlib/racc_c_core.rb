require "racc/static"
require "racc/parser"

grammar = <<~'GRAMMAR'
  class Calculator
    prechigh
      left '*' '/'
      left '+' '-'
    preclow
  rule
    target: expression
    expression: expression '+' expression { result = val[0] + val[2] }
              | expression '-' expression { result = val[0] - val[2] }
              | expression '*' expression { result = val[0] * val[2] }
              | expression '/' expression { result = val[0] / val[2] }
              | '(' expression ')'        { result = val[1] }
              | NUMBER
  end
  ---- inner
    def parse(text)
      @tokens = text.scan(%r{\d+|[-+*/()]}).map do |token|
        token.match?(/\d/) ? [:NUMBER, token.to_i] : [token, token]
      end
      do_parse
    end

    def next_token
      @tokens.shift || [false, false]
    end
GRAMMAR

parsed = Racc::GrammarFileParser.new.parse(grammar, "calculator.y")
states = Racc::States.new(parsed.grammar).nfa
source = Racc::ParserFileGenerator.new(states, parsed.params.dup).generate_parser
eval(source, binding, "calculator.tab.rb")

p Racc::Parser::Racc_Runtime_Type
calculator = Calculator.new
p calculator.parse("2 + 3 * 4")
p calculator.parse("(2 + 3) * 4 - 10 / 5")
begin
  calculator.parse("2 + * 3")
rescue Racc::ParseError => error
  p error.message.lines.first.strip
end
