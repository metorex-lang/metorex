# raise takes an assignment as its argument and raises what it assigns, as
# nokogiri's encoding reader does.
class EncodingFound < StandardError
  def initialize(encoding) = super("found #{encoding}")
end
class Reader
  attr_reader :encoding_found
  def detect(encoding)
    raise @encoding_found = EncodingFound.new(encoding)
  rescue EncodingFound => found
    found.message
  end
end
reader = Reader.new
p reader.detect("UTF-8"), reader.encoding_found.class
begin
  raise error = ArgumentError.new("bad ledger")
rescue => caught
  p [caught.message, error.equal?(caught)]
end
