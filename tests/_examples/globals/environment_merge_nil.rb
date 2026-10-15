# Merging a name into ENV with nil removes it, and so does a block that
# answers nil for a name both sides hold.
ENV["METOREX_MERGE_KEPT"] = "kept"
ENV["METOREX_MERGE_GONE"] = "gone"
ENV["METOREX_MERGE_BLOCK"] = "held"
ENV.merge!("METOREX_MERGE_GONE" => nil, "METOREX_MERGE_ADDED" => "added")
p(ENV["METOREX_MERGE_KEPT"])
p(ENV.key?("METOREX_MERGE_GONE"))
p(ENV["METOREX_MERGE_ADDED"])
ENV.update("METOREX_MERGE_BLOCK" => "given") { |_name, _held, _given| nil }
p(ENV.key?("METOREX_MERGE_BLOCK"))
ENV.merge!("METOREX_MERGE_NEVER_SET" => nil)
p(ENV.key?("METOREX_MERGE_NEVER_SET"))
