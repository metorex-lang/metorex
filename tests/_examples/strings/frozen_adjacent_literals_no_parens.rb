# frozen_string_literal: true

def note text, category:
  [text, category]
end

joined = "maintenance window " \
  "moves to 02:00"
p joined
p joined.frozen?
p note "rates load " \
       "from the cache",
       category: :deprecated
hour = 2
p "window " "at #{hour}:00"
p ("window " "at #{hour}:00").frozen?
p "at #{hour}:00" " sharp"
