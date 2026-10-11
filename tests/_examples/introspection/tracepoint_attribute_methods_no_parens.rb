class Rate
  attr_reader :amount
  attr_writer :region
  attr_accessor :currency

  def initialize
    @amount = 1
  end

  def total = amount * 2
end

rate = Rate.new
events = []
tracepoint = TracePoint.new :call, :c_call, :return, :c_return do |trace|
  events << [trace.event, trace.method_id] if %i[total amount region= currency currency=].include? trace.method_id
end
tracepoint.enable do
  rate.total
  rate.region = :west
  rate.currency = :usd
  rate.currency
end
events.each { |event| p event }
