class Record
  protected

  def guarded
  end
end

class Setup
  def run(klass)
    apply(klass)
  end

  private

  def apply(klass)
    klass.attr_reader(*[])
    klass.attr_writer(*[])
    klass.attr_accessor(*[])
    klass.attr_reader(:title_confirmation)
    klass.attr_writer(:title_confirmation)
  end
end

p Setup.new.run(Record)
record = Record.new
record.title_confirmation = "ok"
p record.title_confirmation
p Record.public_method_defined?(:title_confirmation=)
p record.respond_to?(:guarded)
p record.respond_to?(:guarded, true)

class LookupFailure < NoMethodError
  def initialize(record, attribute)
    @record = record
    super("unknown attribute '#{attribute}' for #{@record.class}.")
  end
end

begin
  raise LookupFailure.new(record, "height")
rescue NoMethodError => error
  p error.message
end

options = { 'namespaced/email': true, if: :x }
p options.keys

module Answering
  def respond_to?(method, include_private_methods = false)
    if super
      true
    elsif !include_private_methods && super(method, true)
      false
    else
      :fallback
    end
  end
end

class Answered
  include Answering

  protected

  def protected_method
  end
end

p Answered.new.respond_to?(:protected_method)
p Answered.new.respond_to?(:protected_method, true)
p Answered.new.respond_to?(:missing)
