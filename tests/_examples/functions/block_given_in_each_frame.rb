class Report
  def plain
    [block_given?, send(:block_given?), iterator_free]
  end

  def iterator_free
    block_given?
  end

  def with_block
    [block_given?, iterator_free, [1].map { block_given? }.first, yield]
  end

  def keeps_block_for_later(&block)
    -> { block_given? }
  end

  def yields_without_a_block
    yield
  rescue LocalJumpError => error
    error.message
  end

  define_method(:defined_with_a_block) { block_given? }
end

report = Report.new
p report.plain
p report.with_block { :yielded }
p report.keeps_block_for_later { :later }.call
p report.keeps_block_for_later.call
p report.yields_without_a_block
p report.defined_with_a_block
p report.defined_with_a_block { :ignored }
p block_given?
