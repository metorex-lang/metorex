# An encoding matches a pattern of encoding constants joined with `|`, as
# net-imap checks `data.encoding in Encoding::ASCII | Encoding::UTF_8`.
def acceptable?(text)
  text.encoding in Encoding::ASCII | Encoding::UTF_8
end

p acceptable?("ledger")
p acceptable?("ledger".encode("US-ASCII"))
p acceptable?("ledger".encode("ISO-8859-1"))
p Encoding::ASCII.equal?(Encoding::US_ASCII)
