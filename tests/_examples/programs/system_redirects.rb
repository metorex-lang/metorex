require "tmpdir"

Dir.mktmpdir do |directory|
  log = File.join(directory, "make.out")
  File.open(log, "w") do |output|
    system("echo", "compiling rates", out: output, err: output)
    system("echo linking rates >&2", out: output, err: output)
    Dir.chdir(directory) { system("echo", "installed", out: output) }
  end
  puts(File.read(log))
end

reader, writer = IO.pipe
p(system("echo", "piped", out: writer))
writer.close
p(reader.read)
p(system("false", out: File::NULL))
p($?.exitstatus)
p(system("pwd", chdir: "/", out: File::NULL))
begin
  system("false", exception: true, out: File::NULL)
rescue RuntimeError => error
  p(error.class)
end
