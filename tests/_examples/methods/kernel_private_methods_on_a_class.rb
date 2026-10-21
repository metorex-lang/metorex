class Runner
end

%i[at_exit trap exit!].each do |name|
  puts "#{name} #{Runner.respond_to?(name)} #{Runner.respond_to?(name, true)} #{Runner.private_methods.include?(name)}"
end
