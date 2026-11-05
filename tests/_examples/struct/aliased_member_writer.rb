# A Struct can alias its [] and []= and replace a member's writer with one
# that writes through the alias. The aliases keep Struct's own methods after
# [] and []= are written again.
Request = Struct.new(:verb, :params) do
  alias_method :member_get, :[]
  private :member_get
  alias_method :member_set, :[]=
  private :member_set

  remove_method :params=
  def params=(hash)
    if params
      params.replace hash
    else
      member_set(:params, hash)
    end
  end

  def first_param
    member_get(:params).first
  end

  def [](key)
    params[key]
  end

  def []=(key, value)
    params[key] = value
  end
end

request = Request.new(:get)
request.params = { "a" => 1 }
p request.params
kept = request.params
request.params = { "b" => 2 }
p request.params, kept.equal?(request.params)
p request.first_param
request["c"] = 3
p request["c"], request.params
p Request.private_instance_methods(false).sort
