class Ledger
  def own_site
    caller_locations(0, 1).first
  end

  def own_entry
    caller(0, 1).first
  end
end

module Kernel
  def kernel_site(up_level: 0)
    caller_locations(up_level, 1).first
  end
end

class Binding
  alias site_of_break kernel_site
end
