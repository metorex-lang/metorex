# ENV is shareable, so a Ractor other than the main one reads it.
Warning[:experimental] = false
ENV["METOREX_RACTOR_EXAMPLE"] = "set"
p Ractor.shareable?(ENV)
p Ractor.new { [ENV["METOREX_RACTOR_EXAMPLE"], ENV.key?("METOREX_NOT_SET")] }.value
p Ractor.make_shareable(ENV).equal?(ENV)
p ENV.frozen?
