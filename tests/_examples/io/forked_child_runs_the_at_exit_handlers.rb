at_exit do
  code = nil
  at_exit { exit code }
  code = 0
  Process.waitpid(fork { exit 42 })
  p $?.exitstatus
  Process.waitpid(fork { at_exit { exit 3 }; exit 42 })
  p $?.exitstatus
end
