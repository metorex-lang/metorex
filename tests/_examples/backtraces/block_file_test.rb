require_relative "block_file_runner"

def short(path)
  File.basename(path.to_s)
end

location = BlockFileRunner.call_block { BlockFileRunner.locate }
puts "call: #{short(location.path)}:#{location.lineno}"

location = BlockFileRunner.exec_block { BlockFileRunner.locate }
puts "instance_exec: #{short(location.path)}:#{location.lineno}"

begin
  BlockFileRunner.exec_block { raise "boom" }
rescue => error
  puts "raised: #{error.backtrace.first.sub(/\A.*\//, '')}"
end

location = BlockFileRunner.module_exec_block { BlockFileRunner.locate }
puts "module_exec: #{short(location.path)}:#{location.lineno}"

location = BlockFileRunner.class_exec_block { BlockFileRunner.locate }
puts "class_exec: #{short(location.path)}:#{location.lineno}"

begin
  BlockFileRunner.module_exec_block { raise "boom" }
rescue => error
  puts "module_exec raised: #{error.backtrace.first.sub(/\A.*\//, '')}"
end
