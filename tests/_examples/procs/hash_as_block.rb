active = { skip_test: [], api: [] }
reasons = [:skip_test, :devcontainer, :api]

p reasons.select(&active)
p reasons.map(&active)
p reasons.any?(&active)
p active.to_proc.call(:api)
p active.to_proc.lambda?
