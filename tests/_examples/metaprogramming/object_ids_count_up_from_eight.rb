first = Object.new
second = Object.new
p first.object_id.class
p second.object_id - first.object_id
p first.object_id == first.object_id
p Object.new.object_id > second.object_id
p Thread.current.object_id < 1000
p nil.object_id, true.object_id, false.object_id, 1.object_id, 0.object_id
