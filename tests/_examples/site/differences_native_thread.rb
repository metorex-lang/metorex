waiting = Thread.new { sleep 0.2 }
sleep 0.05
p Thread.current.native_thread_id.is_a?(Integer)
p waiting.native_thread_id.class
