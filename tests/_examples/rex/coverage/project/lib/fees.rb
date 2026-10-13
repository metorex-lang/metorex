module Fees
  def self.for(amount)
    case amount
    when 0..100 then 1
    else 5
    end
  end
end
