# The receiver a TracePoint reports for the events a method's body fires,
# and for TracePoint#disable.
class Greeter
  def greet
    :hello
  end
end

greeter = Greeter.new
seen = []
trace = TracePoint.new(:call, :line, :return) { |tp| seen << [tp.event, tp.method_id, tp.self.class] }
trace.enable
greeter.greet
trace.disable
seen.each { |event| p(event) }
