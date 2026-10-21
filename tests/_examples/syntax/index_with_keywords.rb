# `[]` takes keywords inside its brackets as any call does, as net-imap
# writes `SearchResult[1, 2, modseq: 1234]`.
class Batch
  def self.[](*numbers, modseq: nil) = [numbers, modseq]
end

p Batch[1, 2, 3, modseq: 1234]
p Batch[
  5, 8, modseq: 99
]
p Batch[13]
