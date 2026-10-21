# `Gem::URI uri` calls the method Gem::URI with an argument, as RubyGems
# names its URI parser, while `Gem::URI` alone is the constant.
module Ledger
  module Entry
    def self.name_of(held) = "entry #{held}"
  end

  def self.Entry(held) = "made from #{held}"
end

held = 1042
made = Ledger::Entry held
p made
p Ledger::Entry("1043")
p Ledger::Entry
