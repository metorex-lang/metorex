def show *values, region: nil, &block
  [values, region, block&.call]
end

def hand_over rates
  yield _ = rates
end

rate = 4
found = case
        when doubled = rate * 2
          doubled
        end
p found
p(case rate
  when limit = 4 then limit + 1
  end)

decl = case entry = [1, 2]
       when Array then entry.size
       end
p [decl, entry]

version = "1.2"
picked = if version and parsed = version.to_f
  parsed
end
p picked

p show 1, region: _ = :west
reload = -> { :reloaded }
p show(&_ = reload)
p show *_ = [5, 6]
p hand_over([7]) { |held| held }

out, err = _ = ["shipped", "none"]
p [out, err]
cache = {}
first, second = cache[:rates] ||= [8, 9]
p [first, second, cache]
