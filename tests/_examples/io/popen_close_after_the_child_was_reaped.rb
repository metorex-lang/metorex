io = IO.popen(["ruby", "-e", "puts :ready"])
p io.gets
_, status = Process.wait2(io.pid)
p status.exitstatus
p io.close
p io.closed?
