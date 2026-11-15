# syswrite warns of buffered IO only while a write is still held in the
# buffer: not after a write on a synced stream, and not after a flush.
reader, writer = IO.pipe
writer.sync = true
writer.write "synced "
writer.syswrite "raw "
writer.sync = false
writer.write "held "
writer.flush
writer.syswrite "after flush"
writer.close
p reader.read
$stderr = STDOUT
STDOUT.sync = true
print "printed\n"
STDOUT.syswrite "syswritten\n"
