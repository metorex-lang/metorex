# A block written after a subscript is handed to the `[]` method.
class Ledger
  def self.[] index, &block
    block ? block.call(index) : index
  end
end

doubled = Ledger[3] { |index| index * 2 }
p doubled
plain = Ledger[4]
p plain
added = Ledger[5] do |index| index + 1 end
p added

# A `do` after a loop's condition or collection opens the loop body.
readings = [[3, 4]]
for reading in readings[0] do p reading end
index = 0
while readings[0][index] do index += 1 end
p index

# A semicolon ends a statement, so a loop keyword after one starts a loop.
counted = 0; until counted == 2 do counted += 1 end; p counted
