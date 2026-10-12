# The innermost frame `caller` and `caller_locations` report names the file
# the running method was written in, not the file that called it.
require_relative "source_file_lib/caller_sites"

def described location
  [File.basename(location.path), location.lineno, location.label]
end

ledger = Ledger.new
p described(ledger.own_site)
p ledger.own_entry.sub(/\A.*\//, "")
p described(kernel_site)
p described(binding.site_of_break)
p described(kernel_site up_level: 1)
