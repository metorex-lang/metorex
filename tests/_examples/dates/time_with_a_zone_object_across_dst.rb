# A Time made with a zone object reads its offset from the zone again when
# arithmetic moves it, so it crosses the end of daylight saving time.
class EasternZone
  ENDS = Time.utc(2005, 10, 30, 6)

  def name = "Eastern"

  def utc_to_local(time)
    time + (time.to_i < ENDS.to_i ? -4 * 3600 : -5 * 3600)
  end

  # The first of the two readings of the hour the clock passes twice.
  def local_to_utc(time)
    wall = Time.utc(time.year, time.month, time.day, time.hour, time.min, time.sec)
    wall + (wall < Time.utc(2005, 10, 30, 2) ? 4 * 3600 : 5 * 3600)
  end
end

zone = EasternZone.new
one_am = Time.new(2005, 10, 30, 1, 0, 0, zone)
p [one_am, one_am.utc_offset, one_am.zone.class]
later = one_am + 3600
p [later, later.utc_offset]
earlier = Time.new(2005, 10, 30, 2, 0, 0, zone) - 3600
p [earlier, earlier.utc_offset]
p (earlier - 3600).utc_offset
p Time.at(EasternZone::ENDS.to_i, in: zone).utc_offset
p Time.at(EasternZone::ENDS.to_i - 1, in: zone).utc_offset
