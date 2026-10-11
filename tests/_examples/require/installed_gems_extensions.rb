require "freight_meter"

p(FreightMeter::NATIVE_VERSION)
p(FreightMeter.with_base([2, 3]))
p(FreightMeter.any_heavy?([5, 150]))
p(FreightMeter.relabel("pallet"))
p(FreightMeter::TAG)
p(Gem.loaded_specs.keys.sort)
built = Dir.glob(File.join(Dir.home, ".gem", "metorex", "extensions", "*", "*", "freight_meter-1.0.0", "freight_meter", "meter.*"))
p(built.size)
