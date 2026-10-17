# A reference that does not by itself keep an object alive. Metorex frees an
# object when the last reference to it goes, and a WeakRef holds a handle
# that names the object without counting as one of those references.

require 'delegate'

class WeakRef < Delegator
  # Raised when the object a reference was made for is gone.
  class RefError < StandardError
  end

  def initialize(object)
    @weakref_handle = nil
    super
  end

  def __getobj__
    held = @weakref_handle.nil? ? nil : __weak_target__(@weakref_handle)
    if held.nil?
      raise RefError, "Invalid Reference - probably recycled"
    end
    held
  end

  def __setobj__(object)
    @weakref_handle = __weak_reference__(object)
  end

  # true while the object is still there to be reached, and nil once it is
  # gone, as MRI answers.
  def weakref_alive?
    return nil if @weakref_handle.nil? || __weak_target__(@weakref_handle).nil?

    true
  end
end
