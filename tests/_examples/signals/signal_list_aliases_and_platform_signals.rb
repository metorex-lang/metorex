p Signal.list["IOT"] == Signal.list["ABRT"]
p Signal.signame(Signal.list["ABRT"])
darwin = RUBY_PLATFORM.include?("darwin")
p darwin ? Signal.list.key?("INFO") && Signal.list.key?("EMT") : true
p darwin ? Signal.signame(Signal.list["INFO"]) : "INFO"
