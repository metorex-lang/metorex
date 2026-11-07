def around_each
  yield :example
end

ran = []
around_each do |example|
  copied = [1].dup
  Dir.chdir("/") { ran << example }
ensure
  ran << :ensured
end
p ran

require "pathname"
p File.binread(Pathname.new(__FILE__)).start_with?("def around_each")

ENV_HOLDER = ENV
Object.send(:remove_const, :ENV)
Object.const_set(:ENV, { "STUBBED" => "1" })
p ENV["STUBBED"]
Object.send(:remove_const, :ENV)
Object.const_set(:ENV, ENV_HOLDER)
p ENV.key?("PATH")
