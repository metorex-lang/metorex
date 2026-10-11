def count_rates(list)
  unless list && !list.empty?
    return # nothing to count
  end
  list.size
end
p([count_rates([]), count_rates([4])])

def last_part(name)
  name.split(/::/).last or raise # (A::B) => B
end
p(last_part("Shipping::Rates"))

def pair_with(*values) = values
held = 1
p(pair_with(held || raise, 2))

table = { rates: [4, 9] }
frame = [2, "warehouse"]
label =
  case frame.size
  when 2
    entry = table[:rates] or raise
    level, location = frame
    "#{level} #{location} #{entry.size}"
  end
p(label)

matched = case [:const, "Rates", [1, 2]]
in [:call, name]
  name
in [:const, name, list]
  if list
    list.select(&:odd?)
  else
    []
  end
end
p(matched)

change = { range: { start: { line: 1 }, end: { line: 3 } }, text: "rates" }
change => {
  range: {
    start: { line: start_row },
    end:   { line: end_row  }
  },
  text: new_text
}
p([start_row, end_row, new_text])

module Kernel
  def rates_marker = :marker
  alias rates_hidden rates_marker if ENV["NO_SUCH_SETTING_HERE"]
  alias rates_kept rates_marker unless ENV["NO_SUCH_SETTING_HERE"]
end
p([respond_to?(:rates_hidden, true), rates_kept])

names = { globals:%w[ globals global_variables ], ivars: %w[ ivars ] }
p(names)

pairs = ->(kind) do
  %i[a b].to_h { [_1.to_s, kind] }
end
p(pairs.(:nil))
p(-> { [1].map { _1 * 2 } }.())
