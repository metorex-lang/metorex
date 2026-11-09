record = Object.new
class << record
  def load_task; end
  private def helper; end
end
p record.private_methods(false).sort
p record.public_methods(false).sort
