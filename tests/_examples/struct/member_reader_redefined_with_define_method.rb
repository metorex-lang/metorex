# A Struct member's reader redefined on the class answers in place of the
# member, and the member reads again once the method is aliased back.
Pallet = Struct.new(:label)
pallet = Pallet.new(:bar)

Pallet.class_exec do
  alias_method :__original_label, :label
  define_method(:label) { :baz }
end
p pallet.label

Pallet.class_exec do
  alias_method :label, :__original_label
  remove_method :__original_label
end
p pallet.label
p pallet.to_a
