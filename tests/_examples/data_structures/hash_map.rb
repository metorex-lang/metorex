prices = {apple: 2, pear: 3}

p prices.map { |name, price| "#{name}:#{price}" }
p prices.collect { |name, price| price * 2 }
p({}.map { |name, price| name })

p prices.map.class, prices.map.size
