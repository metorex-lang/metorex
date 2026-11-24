module Arel
  class HomogeneousInTest
    STRING_TYPE = :string

    def fake_caster
      Object.new.tap do |caster|
        def caster.type_for_attribute(name)
          [name, STRING_TYPE]
        end
      end
    end

    def plain_caster
      caster = Object.new
      def caster.kind
        STRING_TYPE
      end
      caster
    end
  end
end

test = Arel::HomogeneousInTest.new
p test.fake_caster.type_for_attribute(:title)
p test.plain_caster.kind
