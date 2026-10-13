module Outer
  module Helper
    def self.label
      "from Helper"
    end
  end

  class Builder
    class << self
      def define_named(name)
        define_singleton_method(name) { |*arguments| [name, Helper.label, *arguments] }
      end
    end
  end

  module Exposing
    def self.expose(name)
      define_method(name) { |*arguments| Outer::Builder.__send__(name, *arguments) }
      Outer.singleton_class.__send__(:define_method, name, instance_method(name))
    end
  end
end

Outer::Builder.define_named(:feature)
p Outer::Builder.feature(1)
Outer::Exposing.expose(:feature)
p Outer.feature(2)
module Outer
  module Helper
    def self.label = "found"
  end

  class Builder
    class << self
      def plain_read = Helper.label
      def block_read = [1].map { Helper.label }.first
      def define_named(name)
        define_singleton_method(name) { Helper.label }
      end
      def define_with_method(name)
        define_method(name) { Helper.label }
      end
    end

    def self.self_dot_define(name)
      define_singleton_method(name) { Helper.label }
    end
  end
end

p Outer::Builder.plain_read
p Outer::Builder.block_read
Outer::Builder.self_dot_define(:via_self_dot)
p((Outer::Builder.via_self_dot rescue $!.message))
Outer::Builder.define_named(:via_class_self)
p((Outer::Builder.via_class_self rescue $!.message))
Outer::Builder.define_with_method(:instance_one)
p((Outer::Builder.new.instance_one rescue $!.message))
