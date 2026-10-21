class Report; end

class Hooks
  def initialize(owner)
    @owner = owner
  end
end

def make_report_with_hooks
  report = Report.new
  report.singleton_class.instance_variable_set(:@hooks, Hooks.new(report.singleton_class))
  report.instance_variable_set(:@title, "quarterly")
  nil
end

kept = Report.new
kept.singleton_class
make_report_with_hooks
GC.start
p ObjectSpace.each_object(Report).count
p ObjectSpace.each_object(Hooks).count
p kept.singleton_class.attached_object.equal?(kept)
