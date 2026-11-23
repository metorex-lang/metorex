[
  [2026, 90, 0],
  [2026, 13, 1],
  [2026, 0, 1],
  [2026, 12, 0],
  [2026, 12, 32],
  [2026, 16, 1],
  [2026, 15, 1],
  [2026, 1, 1, 25],
  [2026, 1, 1, 24, 1],
  [2026, 1, 1, 1, 60],
  [2026, 1, 1, 1, 1, 61],
  [2026, 1, 1, 1, 1, 60],
  [2026, 2, 30, 0, 0, 0],
  [2026, -1, 1],
  [2026, 1, -1]
].each do |arguments|
  p Time.new(*arguments, 0)
rescue ArgumentError => error
  p [arguments, error.message]
end
[[2026, 90, 0], [2026, 13, 1], [2026, 1, 32]].each do |arguments|
  p Time.utc(*arguments)
rescue ArgumentError => error
  p [arguments, error.message]
end
