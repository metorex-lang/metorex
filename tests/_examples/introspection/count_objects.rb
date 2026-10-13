counted = ObjectSpace.count_objects
p counted.keys.first(5)
p counted[:TOTAL] >= counted[:T_OBJECT] + counted[:T_CLASS] + counted[:T_MODULE]
p counted[:T_CLASS].positive?
held = { stale: 1 }
p ObjectSpace.count_objects(held).equal?(held)
p held.key?(:stale)
p GC.stat(:heap_live_slots).positive?
p [false.object_id, true.object_id, nil.object_id]
