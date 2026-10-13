module Outer
  Local = :outer_local

  def self.run
    BasicObject.class_exec do
      p Class
      p Local
    end
  end
end
Outer.run
