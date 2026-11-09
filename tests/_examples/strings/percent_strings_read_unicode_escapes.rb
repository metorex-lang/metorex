# frozen_string_literal: true
p %("b/\u00e9\\377.rb")
p %(a\u{e9 41}b)
p %Q(\u00e9)
p %W(\u00e9 b)
p "\u{20AC}"
