class EventObjectGroup
end

Listener = Struct.new(:group_class, :delegate)
listeners = [Listener.new(EventObjectGroup, :first)]
groups = listeners.group_by(&:group_class).transform_values { |found| found.map(&:delegate).freeze }.freeze
copy = groups.frozen? ? groups.dup : groups
copy[EventObjectGroup] = copy[EventObjectGroup] + [:second]
p copy.size
p copy.map { |group_class, held| [group_class, held.size] }

grouped = listeners.group_by(&:group_class)
grouped[EventObjectGroup] = []
p grouped.size
transformed = { EventObjectGroup => 1 }.transform_values { |value| value }
transformed[EventObjectGroup] = 2
p transformed.size
