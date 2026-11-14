# An alias a reopened NilClass, TrueClass or FalseClass gives one of its
# native methods answers for nil, true and false.
class NilClass
  alias blank_value? nil? unless method_defined? :blank_value?
  alias shown inspect
end
class TrueClass
  alias shown inspect
end
class FalseClass
  alias shown to_s
end
p nil.blank_value?
p ENV["METOREX_EXAMPLE_UNSET_VARIABLE"].blank_value?
p [nil.shown, true.shown, false.shown]
p nil.send(:shown), nil.method(:shown).call
