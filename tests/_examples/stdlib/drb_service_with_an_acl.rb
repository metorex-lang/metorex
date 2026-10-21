require "drb/drb"
require "drb/acl"

class Ledger
  def balance = 75
end

DRb.install_acl ACL.new(%w[deny all allow localhost allow 127.0.0.1 allow ::1])
service = DRb.start_service(nil, Ledger.new)
puts service.uri.match?(/\Adruby:\/\/.+:\d+\z/)
puts DRbObject.new_with_uri(service.uri).balance
service.stop_service
puts service.alive?
