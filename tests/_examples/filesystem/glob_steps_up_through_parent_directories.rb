here = File.basename(__dir__)
p Dir["#{__dir__}/../#{here}/glob_steps_up_*.rb"].map { |path| path.delete_prefix(__dir__) }
Dir.chdir(__dir__) do
  p Dir["../#{here}/glob_steps_up_*"]
end
