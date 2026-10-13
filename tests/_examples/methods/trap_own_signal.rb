received = []
trap("INT") { received << :interrupted }
Process.kill("INT", Process.pid)
sleep 0.1
p received
