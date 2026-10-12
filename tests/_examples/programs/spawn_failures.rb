# A command that cannot be started raises, and `$?` reports the 127 its
# child ended with.
begin
  Process.spawn("no-such-meter-tool")
rescue Errno::ENOENT => problem
  p problem.message
end
p $?.exitstatus

# A directory to work from that does not exist is the one reported.
begin
  Process.spawn("true", chdir: "no-such-directory")
rescue Errno::ENOENT => problem
  p problem.message
end

# A child started later does not see a variable taken out of ENV.
ENV["METER_ZONE"] = "4"
ENV.delete("METER_ZONE")
p IO.popen(["sh", "-c", "echo ${METER_ZONE:-unset}"], &:read)
ENV["METER_ZONE"] = "5"
ENV.delete_if { |name, _| name == "METER_ZONE" }
p IO.popen(["sh", "-c", "echo ${METER_ZONE:-unset}"], &:read)

# `unsetenv_others:` starts the child with only the variables named.
pid = Process.spawn({ "METER_ZONE" => "6" }, "/bin/sh", "-c", "exit ${METER_ZONE:-1}",
                    unsetenv_others: true, close_others: false)
Process.wait(pid)
p $?.exitstatus
