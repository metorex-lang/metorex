# A refinement of Object reaches a String, an Array, and an instance of a
# class of the program's own, as rexml's DClonable refines `dclone`, and a
# refinement of a closer class wins for that class.
module Copying
  refine Object do
    def copied = "copied #{self.class}"
  end

  refine Array do
    def copied = map(&:copied)
  end
end

using Copying

class Invoice; end

p "total".copied
p Invoice.new.copied
p [1, "a"].copied
