class QuietError < ArgumentError
  def backtrace(*)
    []
  end
end

raise QuietError, "Could not load test file: test/models/accnt.rb.\n\nDid you mean?  test/models/account_test.rb"
