# `include` written in a block calls `self`'s own: run through
# `instance_eval` on `main` from inside a method of another class, it
# includes into Object rather than into the class the method belongs to.
module Labels
  def label = "crate"
end

class Loader
  def run(main)
    main.instance_eval do
      include Labels
    end
  end
end

Loader.new.run(self)
p Object.include?(Labels)
p Loader.instance_method(:label).owner
puts label
