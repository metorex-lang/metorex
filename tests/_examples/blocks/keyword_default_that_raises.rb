# A block keyword default that raises passes the error to the caller.
class Holder
  def secret = (raise ArgumentError, "missing")
  def maker
    proc { |salt, key: self.secret| [salt, key] }
  end
  def lam
    lambda { |salt, key: self.secret| [salt, key] }
  end
end
h = Holder.new
p (h.maker.call(1, **{}) rescue $!)
p (h.maker.call(1) rescue $!)
p (h.maker.call(1, key: 2) rescue $!)
p (h.lam.call(1, **{}) rescue $!)
gen = h.maker
p (gen.call("salt", **{}) rescue $!)
opts = {}
p (gen.call("salt", **opts) rescue $!)
