p `missing_program_for_this_example -v 2>&1`.include?("not found")
p $?.exitstatus

begin
  `missing_program_for_this_example`
rescue SystemCallError => error
  p [error.class, error.message]
end

`exit 3`
p $?.exitstatus
p `REPORT_NAME=nightly env`.include?("REPORT_NAME=nightly")
p `echo  two   words`
