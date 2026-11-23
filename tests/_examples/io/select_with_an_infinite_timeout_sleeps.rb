waiting = Thread.new { IO.select(nil, nil, nil, Float::INFINITY) }
Thread.pass while waiting.status && waiting.status != "sleep"
p waiting.status
waiting.kill
waiting.join
p waiting.status
