# A trace hears String() and nil? on a value in a method body, and not the
# questions String() asks of the core library on the way.
class Holder
  def run
    a = 0
    e = []
    t = TracePoint.new(:c_call, :c_return) { |tp| e << [tp.event, tp.method_id, tp.lineno] }
    t.enable
    x = String(a)
    y = {"a": 1.to_s}.nil?
    t.disable
    e
  end
end
Holder.new.run.each { |i| p i }
