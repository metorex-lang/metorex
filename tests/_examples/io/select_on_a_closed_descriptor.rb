reader, writer = IO.pipe
copy = IO.for_fd(reader.fileno, autoclose: false)
reader.close
begin
  p IO.select([copy], nil, nil, 0.05)
rescue SystemCallError => e
  p e.class
  p e.message
end
writer.close
