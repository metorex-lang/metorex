# Open3's captures take their options as a Hash after the command, written
# as keywords or as a Hash argument, as parser's tests pass `stdin_data`.
require "open3"

output, status = Open3.capture2("cat", stdin_data: "entry one")
p output, status.success?
output, errors, status = Open3.capture3("cat", "-", { stdin_data: "entry two" })
p output, errors, status.success?
output, status = Open3.capture2e("sh", "-c", "cat; echo note >&2", { stdin_data: "entry three\n", binmode: true })
p output, output.encoding, status.success?
