# The date library's reading and writing of times, added to Time itself.
require 'date'

class Time
  def self.parse(text, now = Time.now)
    in_zone(DateTime.parse(text).to_time, text[/([A-Za-z]+|[-+]\d\d:?\d\d)\s*\z/, 1])
  end

  def self.xmlschema(text)
    in_zone(DateTime.iso8601(text).to_time, text[/(Z|[-+]\d\d:?\d\d)\s*\z/i, 1])
  end

  def self.rfc2822(text)
    in_zone(DateTime.rfc2822(text).to_time, text.split.last)
  end

  class << self
    alias rfc822 rfc2822
    alias iso8601 xmlschema
  end

  # The time `text` names read with `format`. A field the text leaves out
  # ahead of the first one it gives comes from `now`, and the rest start at
  # their lowest, as Ruby's own `Time.strptime` reads them.
  def self.strptime(text, format, now = self.now)
    parts = Date._strptime(text, format)
    raise ArgumentError, "invalid date or strptime format - '#{text}' '#{format}'" if parts.nil?
    if (seconds = parts[:seconds])
      seconds += parts[:sec_fraction] if parts[:sec_fraction]
      held = Time.at(seconds)
      return parts[:offset] ? held.localtime(parts[:offset]) : held
    end
    year = parts[:year]
    year = yield(year) if year && block_given?
    mon = parts[:mon]
    day = parts[:mday]
    if parts[:yday] && year
      ordinal = Date.ordinal(year, parts[:yday])
      mon, day = ordinal.mon, ordinal.mday
    end
    hour, min, sec = parts[:hour], parts[:min], parts[:sec]
    fraction = parts[:sec_fraction]
    if [year, mon, day, hour, min, sec, fraction].all?(&:nil?)
      raise ArgumentError, "no time information in #{text.inspect}"
    end
    offset = parts[:offset]
    reference = offset ? now.getlocal(offset) : now.getlocal
    fields = [year, mon, day, hour, min, sec]
    known = [reference.year, reference.mon, reference.day, reference.hour, reference.min, reference.sec]
    first_given = fields.index { |field| !field.nil? } || fields.size
    year, mon, day, hour, min, sec = fields.each_with_index.map do |field, place|
      next field unless field.nil?
      place < first_given ? known[place] : [1970, 1, 1, 0, 0, 0][place]
    end
    sec += fraction if fraction
    return Time.local(year, mon, day, hour, min, sec) if offset.nil?
    made = Time.new(year, mon, day, hour, min, sec, offset)
    zone = text[/(?:\A|\s)(UTC|GMT|UT|Z)\z/i, 1]
    zone ? made.utc : made
  end

  def self.httpdate(text)
    DateTime.httpdate(text).to_time.utc
  end

  # A zone of -0000, -00:00, UTC, UT or Z says the time is in UTC, where an
  # offset such as +0000 names a local zone that happens to sit at UTC.
  def self.in_zone(time, zone)
    return time if zone.nil?
    zone.match?(/\A(?:-00:00|-0000|-00|UTC|Z|UT)\z/i) ? time.utc : time
  end
  private_class_method :in_zone

  def rfc2822
    strftime("%a, %d %b %Y %T ") << (utc? ? "-0000" : strftime("%z"))
  end

  alias rfc822 rfc2822

  def httpdate
    self.getutc.strftime("%a, %d %b %Y %H:%M:%S GMT")
  end
end
