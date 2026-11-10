# A double splat of a Hash subclass instance passes the hash it holds,
# without asking it for to_hash, which a class answering any name through
# method_missing, as ActiveSupport::OrderedOptions does, would answer wrong.
class Options < Hash
  def method_missing(name, *args)
    name.end_with?("=") ? self[name.to_s.chomp("=").to_sym] = args.first : self[name]
  end
  def respond_to_missing?(*) = true
end
def configure(primary_key: nil, **properties) = [primary_key, properties]
options = Options.new
options.store_key_references = true
p configure(primary_key: "k", **options)
class Plain < Hash; end
plain = Plain.new
plain[:mode] = :strict
p configure(**plain)
