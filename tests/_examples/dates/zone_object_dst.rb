# A zone object with a daylight period from April through October.
class SampleZone
  def initialize(name)
    @name = name
  end

  def daylight?(time)
    (4..10).cover?(time.month)
  end

  def utc_to_local(time)
    time + (daylight?(time) ? -14400 : -18000)
  end

  def local_to_utc(time)
    time - (daylight?(time) ? -14400 : -18000)
  end

  def dst?(time)
    daylight?(time)
  end

  def abbr(time)
    daylight?(time) ? "SDT" : "SST"
  end

  def name
    @name
  end
end

zone = SampleZone.new("Sample")
[Time.new(2000, 1, 1, 1, in: zone), Time.new(2000, 6, 1, 1, in: zone),
 Time.at(959835600, in: zone), Time.utc(2000, 6, 1).getlocal(zone)].each do |time|
  p [time.iso8601, time.utc_offset, time.strftime("%Z"), time.isdst, time.zone.equal?(zone)]
end
