class Report
  def summary; end
end
Report.send(:undef_method, :summary)
report = Report.new
begin
  report.method(:summary)
rescue NameError => error
  p error.message
end
p report.respond_to?(:summary)
class Report
  def title; end
  private def draft; end
end
Report.send(:undef_method, :draft)
p report.public_methods(false), report.private_methods(false)
