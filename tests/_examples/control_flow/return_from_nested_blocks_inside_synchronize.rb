# A `return` inside blocks nested in Mutex#synchronize leaves the method with
# its value, past a `break` and `next` in the inner block, and through a
# block handed on with `&block`, as Zeitwerk's check for a directory another
# loader manages does.
class Owners
  def initialize(owners)
    @owners = owners
  end

  def each(&block)
    @owners.each(&block)
  end
end

module Registry
  @mutex = Mutex.new
  @owners = Owners.new([[:first, ["/app", "/lib"]], [:second, ["/app/models"]]])

  class << self
    attr_reader :owners

    def owner_of(directory, skipping)
      @mutex.synchronize do
        owners.each do |owner, roots|
          next if owner == skipping

          roots.each do |root|
            next if !directory.start_with?(root) && !root.start_with?(directory)
            break if root == "/never"

            return owner
          end
        end

        nil
      end
    end
  end
end

p Registry.owner_of("/app", :second)
p Registry.owner_of("/app/models/user", :first)
p Registry.owner_of("/other", :first)
p Registry.owner_of("/app", :first)
