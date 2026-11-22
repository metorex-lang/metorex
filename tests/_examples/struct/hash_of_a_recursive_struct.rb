Car = Struct.new(:make, :model, :year)
car = Car.new("Honda", "Accord", "1998")
similar_car = Car.new("Honda", "Accord", "1998")
car[:make] = car
similar_car[:make] = car
p car.hash == similar_car.hash
p car.eql?(similar_car)
p car.hash == Car.new(1, 2, 3).hash
plain = Car.new("Honda", "Accord", "1998")
p plain.hash == Car.new("Honda", "Accord", "1998").hash
p plain.hash == Car.new("Honda", "Civic", "1998").hash
p({ car => 1 }[similar_car])
