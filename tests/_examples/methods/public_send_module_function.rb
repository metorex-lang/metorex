# A module function answers `public_send` on its module, where its copy is
# public, as net-imap's tests call NumValidator's.
module NumberCheck
  module_function

  def valid_number?(held) = held.between?(0, 0xffff_ffff)
end

p NumberCheck.public_send(:valid_number?, 42)
p NumberCheck.valid_number?(-1)
p NumberCheck.respond_to?(:valid_number?)
