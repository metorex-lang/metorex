class Shelf
  def [](name, *rest, **options)
    [name, rest, options]
  end
end

shelf = Shelf.new
options = { unit: "kg" }
none = {}
sizes = [2, 3]

p shelf["crate", **options]
p shelf["crate", **none]
p shelf[*sizes]
p shelf["crate", *sizes, **options]
p shelf["crate", *sizes, scale: 2, **options]
