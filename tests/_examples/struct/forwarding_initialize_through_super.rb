class Fetched < Struct.new(:seqno, :attr)
  def initialize(...)
    super
    attr and
      attr_uid = attr["UID"] and
      attr_uid != seqno and
      warn "#{self.class} UIDs do not match (#{attr_uid} != #{seqno})"
  end
end

$stderr = $stdout
Fetched.new(22222, "UID" => 54_321)
p Fetched.new(7, nil).to_a
p Fetched.new(5, "UID" => 5).attr
class Bare < Struct.new(:seqno, :attr)
  def initialize(...) = super
end
class Explicit < Struct.new(:seqno, :attr)
  def initialize(...) = super(...)
end
class Splatted < Struct.new(:seqno, :attr)
  def initialize(*args, **opts) = super
end
p Bare.new(1, "UID" => 2).attr, Explicit.new(1, "UID" => 2).attr, Splatted.new(1, "UID" => 2).attr
p Bare.new(1, uid: 2).attr, Bare.new(1, {"UID" => 2}).attr
def receiver(*args, **opts) = [args, opts]
def forward(...) = receiver(...)
p forward(1, "UID" => 2)
