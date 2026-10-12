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
