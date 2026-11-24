def with_connection
  yield :connection
end

def compute(loaded)
  if loaded
    size = 1
    if size > 0
      timestamp = :loaded
    end
  else
    with_connection do |c|
      size, timestamp = [3, :queried]
      size = 0 unless size
    end
  end
  [size, timestamp]
end

p compute(true)
p compute(false)

def counted
  [1, 2].each { |n| total = n } if false
  [1, 2].each { |n| total = n }
  defined?(total)
end
p counted

def by_case(value)
  case value
  when 1 then size = 1
  when 2 then with_connection { size = 2 }
  else with_connection { size = 3 }
  end
  size
end
p [by_case(2), by_case(5)]

def by_case_value(value)
  result = case value
           when 1 then (size = 1)
           else with_connection { size = 4 }
           end
  [result, size]
end
p by_case_value(9)
