require "drb"
require "drb/unix"
require "tmpdir"

class Tally
  def add(left, right) = left + right
end

Dir.mktmpdir do |directory|
  uri = "drbunix:#{directory}/tally.sock"
  DRb.start_service(uri, Tally.new)
  remote = DRbObject.new_with_uri(uri)
  p remote.add(2, 3)
  p DRb.uri == uri
  DRb.stop_service
end
