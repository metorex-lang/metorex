# Arithmetic, comparison, and indexing on core values reach a method the
# program wrote in place of the built-in one.
def sums(left, right) = [left + right, left - right, left * right, left / right, left % right]
def comparisons(left, right) = [left == right, left != right, left < right, left <= right, left > right, left >= right]
def first(collection, key) = collection[key]

p sums(7, 2)
p sums(7.0, 2.0)
p comparisons(1, 2)
p comparisons(1.5, 1.5)
p first([1, 2], 0)
p first({a: 1}, :a)
p [1, 2].send(:first)

class Integer
  alias_method :original_plus, :+

  def +(other) = original_plus(other).original_plus(100)
  def <(other) = :less
end

class Float
  def *(other) = :times
end

class Array
  def [](index) = :indexed
end

class Hash
  def [](key) = :looked_up
end

module Sending
  def first = :sent
end
Array.prepend(Sending)

p sums(7, 2)
p sums(7.0, 2.0)
p comparisons(1, 2)
p first([1, 2], 0)
p first({a: 1}, :a)
p [1, 2].send(:first)
