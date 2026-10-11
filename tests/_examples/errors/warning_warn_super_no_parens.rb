$stderr = $stdout

module Warning
  def warn *args, **kwargs
    $stdout.print "rewritten: "
    super
  end
end

warn "maintenance window moves to 02:00"
Warning.warn "rates reload at 03:00\n"
