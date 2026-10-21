#!/usr/bin/env ruby
# Run with `bundle exec`, which puts the bundle's gems on the load path.
require "ledger_totals"
p LedgerTotals.sum([1200, 45, 75])
