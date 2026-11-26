module Schema
  class Migration
    def self.[](version)
      version == 5.2 ? Legacy : self
    end

    def self.inherited(subclass)
      super
      puts "#{subclass} inherits from #{subclass.superclass}, told by #{self}"
    end
  end

  class Legacy < Migration; end
end

class CreateUsers < Schema::Migration[5.2]; end
p CreateUsers.superclass

class CreateGroups < Schema::Migration[8.1]
  def self.label = "groups"
end
p CreateGroups.superclass
p CreateGroups.label
