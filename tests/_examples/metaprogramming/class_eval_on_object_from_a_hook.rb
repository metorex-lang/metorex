module Tracking
  def self.included(base)
    Object.class_eval do
      def tracked_note
        frozen? ? {} : (@tracked_note ||= {})
      end

      def tracked_note=(value)
        @tracked_note = value unless frozen?
      end
    end unless Object.method_defined?(:tracked_note)
  end
end

class Backend
  include Tracking
end

text = +"hello"
text.tracked_note = { locale: :en }
p text.tracked_note, [1].tap { |list| list.tracked_note = { size: 1 } }.tracked_note
p "frozen".freeze.tracked_note, 5.tracked_note, :sym.respond_to?(:tracked_note)
p Object.instance_method(:tracked_note).owner
