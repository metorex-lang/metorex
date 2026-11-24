# A calendar date, held as the Julian Day Number it stands for together with
# the day the Gregorian calendar takes over from the Julian one.
class Date
  include Comparable

  class Error < ArgumentError
  end

  # A value that compares greater or less than every date, which is what the
  # calendars that never reform are written as.
  class Infinity < Numeric
    include Comparable

    def initialize(direction = 1)
      @direction = direction <=> 0
    end

    def direction
      @direction
    end

    def zero?
      false
    end

    def finite?
      false
    end

    def infinite?
      @direction == 0 ? nil : @direction
    end

    def nan?
      @direction == 0
    end

    def abs
      Infinity.new
    end

    def -@
      Infinity.new(-@direction)
    end

    def +@
      Infinity.new(@direction)
    end

    def <=>(other)
      return @direction <=> other.direction if other.is_a?(Infinity)
      return @direction if other.is_a?(Numeric)
      nil
    end

    def coerce(other)
      return [other.direction, @direction] if other.is_a?(Infinity)
      [-@direction, @direction]
    end

    def to_f
      @direction * Float::INFINITY
    end
  end

  ITALY = 2299161
  ENGLAND = 2361222
  JULIAN = Infinity.new
  GREGORIAN = Infinity.new(-1)
  MONTHNAMES = [nil, "January", "February", "March", "April", "May", "June",
                "July", "August", "September", "October", "November",
                "December"].each { |name| name.freeze }.freeze
  DAYNAMES = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday",
              "Saturday"].each { |name| name.freeze }.freeze
  ABBR_MONTHNAMES = [nil, "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug",
                     "Sep", "Oct", "Nov", "Dec"].each { |name| name.freeze }.freeze
  ABBR_DAYNAMES = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri",
                   "Sat"].each { |name| name.freeze }.freeze
  VERSION = "3.4.1"

  # ── Building ─────────────────────────────────────────────────────────────

  def self.from_jd(jd, start)
    made = allocate
    made.instance_variable_set(:@jd, jd)
    made.instance_variable_set(:@start, start)
    made
  end

  def self.jd(number = 0, start = ITALY)
    from_jd(number.to_i, start)
  end

  def self.civil(year = -4712, month = 1, day = 1, start = ITALY)
    found = valid_civil_jd(year, month, day, start)
    raise Date::Error, "invalid date" if found.nil?
    from_jd(found, start)
  end

  def self.new(year = -4712, month = 1, day = 1, start = ITALY)
    civil(year, month, day, start)
  end

  def self.ordinal(year = -4712, day = 1, start = ITALY)
    found = valid_ordinal_jd(year, day, start)
    raise Date::Error, "invalid date" if found.nil?
    from_jd(found, start)
  end

  def self.commercial(year = -4712, week = 1, weekday = 1, start = ITALY)
    raise Date::Error, "invalid date" if week < 1 || week > 53
    raise Date::Error, "invalid date" if weekday < 1 || weekday > 7
    fourth = civil(year, 1, 4, start)
    monday = fourth.jd - (fourth.cwday - 1)
    found = monday + (week - 1) * 7 + (weekday - 1)
    made = from_jd(found, start)
    raise Date::Error, "invalid date" unless made.cwyear == year
    made
  end

  # The forms ISO 8601 writes a date, a time, or both in: extended, with
  # separators, and basic, without them.
  ISO8601_EXTENDED_DATETIME = /\A\s*(?:([-+]?\d{2,}|-)-(\d{2})?(?:-(\d{2}))?|([-+]?\d{2,})?-(\d{3})|(\d{4}|\d{2})?-w(\d{2})-(\d)|-w-(\d))(?:t(\d{2}):(\d{2})(?::(\d{2})(?:[,.](\d+))?)?(z|[-+]\d{2}(?::?\d{2})?)?)?\s*\z/i
  ISO8601_BASIC_DATETIME = /\A\s*(?:([-+]?(?:\d{4}|\d{2})|--)(\d{2}|-)(\d{2})|([-+]?(?:\d{4}|\d{2}))(\d{3})|-(\d{3})|(\d{4}|\d{2})w(\d{2})(\d)|-w(\d{2})(\d)|-w-(\d))(?:t?(\d{2})(\d{2})(?:(\d{2})(?:[,.](\d+))?)?(z|[-+]\d{2}(?:\d{2})?)?)?\s*\z/i
  ISO8601_EXTENDED_TIME = /\A\s*(\d{2}):(\d{2})(?::(\d{2})(?:[,.](\d+))?(z|[-+]\d{2}(:?\d{2})?)?)?\s*\z/i
  ISO8601_BASIC_TIME = /\A\s*(\d{2})(\d{2})(?:(\d{2})(?:[,.](\d+))?(z|[-+]\d{2}(\d{2})?)?)?\s*\z/i
  RFC3339_DATETIME = /\A\s*(-?\d{4})-(\d{2})-(\d{2})(?:t|\s)(\d{2}):(\d{2}):(\d{2})(?:\.(\d+))?(z|[-+]\d{2}:\d{2})\s*\z/i
  XMLSCHEMA_DATETIME = /\A\s*(-?\d{4,})(?:-(\d{2})(?:-(\d{2}))?)?(?:t(\d{2}):(\d{2}):(\d{2})(?:\.(\d+))?)?(z|[-+]\d{2}:\d{2})?\s*\z/i
  XMLSCHEMA_TIME = /\A\s*(\d{2}):(\d{2}):(\d{2})(?:\.(\d+))?(z|[-+]\d{2}:\d{2})?\s*\z/i
  XMLSCHEMA_TRUNCATED = /\A\s*(?:--(\d{2})(?:-(\d{2}))?|---(\d{2}))(z|[-+]\d{2}:\d{2})?\s*\z/i

  # The text a reader of written dates is handed, which must be a String.
  def self.written_text(text)
    unless text.is_a?(String)
      raise TypeError, "no implicit conversion of #{text.class} into String"
    end
    text.to_s
  end

  # A year written with fewer than four digits stands in this century or the
  # last, as ISO 8601's two-digit years do.
  def self.written_year(digits)
    year = digits.to_i
    digits.length < 4 ? complete_year(year, true) : year
  end

  def self.sec_fraction_of(digits)
    Rational(digits.to_i, 10**digits.length)
  end

  def self.add_zone(found, written)
    return found if written.nil?

    found[:zone] = written
    found[:offset] = zone_to_diff(written)
    found
  end

  # The time of day and zone that follow a date, from the place in the match
  # the hour sits.
  def self.add_clock(found, match, at)
    return found if match[at].nil?

    found[:hour] = match[at].to_i
    found[:min] = match[at + 1].to_i
    found[:sec] = match[at + 2].to_i unless match[at + 2].nil?
    found[:sec_fraction] = sec_fraction_of(match[at + 3]) unless match[at + 3].nil?
    add_zone(found, match[at + 4])
  end

  def self._iso8601(text, limit: 128)
    return {} if text.nil?
    text = written_text(text)
    if (match = ISO8601_EXTENDED_DATETIME.match(text))
      found = {}
      if match[1]
        found[:mday] = match[3].to_i if match[3]
        found[:year] = written_year(match[1]) unless match[1] == "-"
        if match[2].nil?
          return {} unless match[1] == "-"
        else
          found[:mon] = match[2].to_i
        end
      elsif match[5]
        found[:yday] = match[5].to_i
        found[:year] = written_year(match[4]) if match[4]
      elsif match[8]
        found[:cweek] = match[7].to_i
        found[:cwday] = match[8].to_i
        found[:cwyear] = written_year(match[6]) if match[6]
      elsif match[9]
        found[:cwday] = match[9].to_i
      end
      return add_clock(found, match, 10)
    end
    if (match = ISO8601_BASIC_DATETIME.match(text))
      found = {}
      if match[3]
        found[:mday] = match[3].to_i
        found[:year] = written_year(match[1]) unless match[1] == "--"
        if match[2] == "-"
          return {} unless match[1] == "--"
        else
          found[:mon] = match[2].to_i
        end
      elsif match[5]
        found[:yday] = match[5].to_i
        found[:year] = written_year(match[4])
      elsif match[6]
        found[:yday] = match[6].to_i
      elsif match[9]
        found[:cweek] = match[8].to_i
        found[:cwday] = match[9].to_i
        found[:cwyear] = written_year(match[7])
      elsif match[11]
        found[:cweek] = match[10].to_i
        found[:cwday] = match[11].to_i
      elsif match[12]
        found[:cwday] = match[12].to_i
      end
      return add_clock(found, match, 13)
    end
    [ISO8601_EXTENDED_TIME, ISO8601_BASIC_TIME].each do |pattern|
      match = pattern.match(text)
      return add_clock({}, match, 1) if match
    end
    {}
  end

  def self._rfc3339(text, limit: 128)
    return {} if text.nil?
    match = RFC3339_DATETIME.match(written_text(text))
    return {} unless match

    found = { year: match[1].to_i, mon: match[2].to_i, mday: match[3].to_i,
              hour: match[4].to_i, min: match[5].to_i, sec: match[6].to_i }
    add_zone(found, match[8])
    found[:sec_fraction] = sec_fraction_of(match[7]) if match[7]
    found
  end

  def self._xmlschema(text, limit: 128)
    return {} if text.nil?
    text = written_text(text)
    if (match = XMLSCHEMA_DATETIME.match(text))
      found = { year: match[1].to_i }
      found[:mon] = match[2].to_i if match[2]
      found[:mday] = match[3].to_i if match[3]
      found[:hour] = match[4].to_i if match[4]
      found[:min] = match[5].to_i if match[5]
      found[:sec] = match[6].to_i if match[6]
      found[:sec_fraction] = sec_fraction_of(match[7]) if match[7]
      return add_zone(found, match[8])
    end
    if (match = XMLSCHEMA_TIME.match(text))
      found = { hour: match[1].to_i, min: match[2].to_i, sec: match[3].to_i }
      found[:sec_fraction] = sec_fraction_of(match[4]) if match[4]
      return add_zone(found, match[5])
    end
    if (match = XMLSCHEMA_TRUNCATED.match(text))
      found = {}
      found[:mon] = match[1].to_i if match[1]
      found[:mday] = match[2].to_i if match[2]
      found[:mday] = match[3].to_i if match[3]
      return add_zone(found, match[4])
    end
    {}
  end

  def self.iso8601(text = "-4712-01-01", start = ITALY, limit: 128)
    read_date_fields(text, start) { |held| _iso8601(held) }
  end

  def self.rfc3339(text = "-4712-01-01T00:00:00+00:00", start = ITALY, limit: 128)
    read_date_fields(text, start) { |held| _rfc3339(held) }
  end

  def self.xmlschema(text = "-4712-01-01", start = ITALY, limit: 128)
    read_date_fields(text, start) { |held| _xmlschema(held) }
  end

  # The date the fields a reader found in `text` name.
  def self.read_date_fields(text, start)
    raise Date::Error, "invalid date" if text.nil?
    fields = yield(text)
    new_by_frags(fields, start)
  end

  # A date written as text, read back into the fields it names, which is
  # empty when the text names no date at all.
  def self._parse(text, comp = true, limit: 128)
    check_limit(text, limit)
    text = coerce_to_string(text)
    read_parse(text, comp)
  end

  # Text longer than `limit` is refused before it is read.
  def self.check_limit(text, limit)
    return if text.nil? || limit.nil?
    return unless text.is_a?(String) || text.respond_to?(:to_str)
    length = coerce_to_string(text).length
    return if length <= limit
    raise ArgumentError, "string length (#{length}) exceeds the limit #{limit}"
  end
  private_class_method :check_limit

  PARSE_DAYS = %w[sun mon tue wed thu fri sat].freeze
  PARSE_MONTHS = %w[jan feb mar apr may jun jul aug sep oct nov dec].freeze
  PARSE_ABBR_DAYS = "sun|mon|tue|wed|thu|fri|sat"
  PARSE_ABBR_MONTHS = "jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec"
  PARSE_ERA = "c(?:e|\\.e\\.)|b(?:ce|\\.c\\.e\\.)|a(?:d|\\.d\\.)|b(?:c|\\.c\\.)"
  PARSE_PATTERNS = {
    separators: /[^-+',.\/:@[:alnum:]\[\]]+/,
    day: /\b(#{PARSE_ABBR_DAYS})[^-\/\d\s]*/i,
    time: /((?<!\d)\d+\s*(?:(?::\s*\d+(?:\s*:\s*\d+(?:[,.]\d*)?)?|h(?:\s*\d+m?(?:\s*\d+s?)?)?)(?:\s*[ap](?:m\b|\.m\.))?|[ap](?:m\b|\.m\.)))(?:\s*((?:gmt|utc?)?[-+]\d+(?:[,.:]\d+(?::\d+)?)?|(?-i:[[:alpha:].\s]+)(?:standard|daylight)\stime\b|(?-i:[[:alpha:]]+)(?:\sdst)?\b))?/i,
    clock: /\A(\d+)h?(?:\s*:?\s*(\d+)m?(?:\s*:?\s*(\d+)(?:[,.](\d+))?s?)?)?(?:\s*([ap])(?:m\b|\.m\.))?/i,
    eu: /('?(?<!\d)\d+)[^-\d\s]*\s*(#{PARSE_ABBR_MONTHS})[^-\d\s']*(?:\s*(?:\b(#{PARSE_ERA})(?!(?<!\.)[a-z]))?\s*('?-?\d+(?:(?:st|nd|rd|th)\b)?))?/i,
    us: /\b(#{PARSE_ABBR_MONTHS})[^-\d\s']*\s*('?\d+)[^-\d\s']*(?:\s*+,?\s*+(#{PARSE_ERA})?\s*('?-?\d+))?/i,
    iso: /('?[-+]?(?<!\d)\d+)-(\d+)-('?-?\d+)/,
    iso21: /\b(\d{2}|\d{4})?-?w(\d{2})(?:-?(\d))?\b/i,
    iso22: /-w-(\d)\b/i,
    iso23: /--(\d{2})?-(\d{2})\b/,
    iso24: /--(\d{2})(\d{2})?\b/,
    iso25_guard: /[,.](\d{2}|\d{4})-\d{3}\b/,
    iso25: /\b(\d{2}|\d{4})-(\d{3})\b/,
    iso26_guard: /\d-\d{3}\b/,
    iso26: /\b-(\d{3})\b/,
    jis: /\b([mtshr])(\d+)\.(\d+)\.(\d+)/i,
    vms11: /('?-?(?<!\d)\d+)-(#{PARSE_ABBR_MONTHS})[^-\/.]*-('?-?\d+)/i,
    vms12: /\b(#{PARSE_ABBR_MONTHS})[^-\/.]*-('?-?\d+)(?:-('?-?\d+))?/i,
    sla: /('?-?(?<!\d)\d+)\/\s*('?\d+)(?:\D\s*('?-?\d+))?/i,
    dot: /('?-?(?<!\d)\d+)\.\s*('?\d+)\.\s*('?-?\d+)/i,
    year: /'(\d+)\b/,
    mon: /\b(#{PARSE_ABBR_MONTHS})\S*/i,
    mday: /((?<!\d)\d+)(st|nd|rd|th)\b/i,
    ddd: /([-+]?)((?<!\d)\d{2,14})(?:\s*t?\s*(\d{2,6})?(?:[,.](\d*))?)?(?:\s*(z\b|[-+]\d{1,4}\b|\[[-+]?\d[^\]]*\]))?/i,
    bc: /\b(bc\b|bce\b|b\.c\.|b\.c\.e\.)/i,
    frag: /\A\s*(\d{1,2})\s*\z/i
  }.freeze
  JAPANESE_ERA_OFFSETS = { "m" => 1867, "t" => 1911, "s" => 1925, "h" => 1988, "r" => 2018 }.freeze
  private_constant :PARSE_DAYS, :PARSE_MONTHS, :PARSE_ABBR_DAYS, :PARSE_ABBR_MONTHS,
                   :PARSE_ERA, :PARSE_PATTERNS, :JAPANESE_ERA_OFFSETS

  def self.read_parse(text, comp)
    text = text.gsub(PARSE_PATTERNS[:separators], " ")
    found = { _comp: comp }

    parse_day(text, found) if parse_has?(text, :alpha)
    parse_time(text, found) if parse_has?(text, :digit)
    catch(:parsed) do
      if parse_has?(text, :alpha, :digit)
        throw :parsed if parse_eu(text, found)
        throw :parsed if parse_us(text, found)
      end
      throw :parsed if parse_has?(text, :digit, :dash) && parse_iso(text, found)
      throw :parsed if parse_has?(text, :digit, :dot) && parse_jis(text, found)
      throw :parsed if parse_has?(text, :alpha, :digit, :dash) && parse_vms(text, found)
      throw :parsed if parse_has?(text, :digit, :slash) && parse_sla(text, found)
      throw :parsed if parse_has?(text, :digit, :dot) && parse_dot(text, found)
      throw :parsed if parse_has?(text, :digit) && parse_iso2(text, found)
      throw :parsed if parse_has?(text, :digit) && parse_year(text, found)
      throw :parsed if parse_has?(text, :alpha) && parse_mon(text, found)
      throw :parsed if parse_has?(text, :digit) && parse_mday(text, found)
      throw :parsed if parse_has?(text, :digit) && parse_ddd(text, found)
    end
    parse_bc(text, found) if parse_has?(text, :alpha)
    parse_frag(text, found) if parse_has?(text, :digit)

    if found.delete(:_bc)
      found[:cwyear] = -found[:cwyear] + 1 unless found[:cwyear].nil?
      found[:year] = -found[:year] + 1 unless found[:year].nil?
    end
    if found.delete(:_comp)
      [:cwyear, :year].each do |field|
        held = found[field]
        next if held.nil? || held < 0 || held > 99
        found[field] = held >= 69 ? held + 1900 : held + 2000
      end
    end
    found[:offset] = zone_to_diff(found[:zone]) if !found[:zone].nil? && found[:offset].nil?
    found
  end
  private_class_method :read_parse

  # Whether the text holds a character of each kind named.
  def self.parse_has?(text, *kinds)
    kinds.all? do |kind|
      case kind
      when :alpha then text.match?(/[A-Za-z]/)
      when :digit then text.match?(/[0-9]/)
      when :dash then text.include?("-")
      when :dot then text.include?(".")
      when :slash then text.include?("/")
      end
    end
  end
  private_class_method :parse_has?

  # Matches `pattern` in `text`, blanks out what it matched with a space,
  # and hands the match to the block. False when it does not match.
  def self.parse_sub(text, name)
    matched = PARSE_PATTERNS[name].match(text)
    return false if matched.nil?
    text[matched.begin(0), matched.end(0) - matched.begin(0)] = " "
    yield matched
    true
  end
  private_class_method :parse_sub

  def self.parse_day(text, found)
    parse_sub(text, :day) do |matched|
      found[:wday] = PARSE_DAYS.index(matched[1][0, 3].downcase)
    end
  end
  private_class_method :parse_day

  def self.parse_time(text, found)
    parse_sub(text, :time) do |matched|
      found[:zone] = matched[2] unless matched[2].nil?
      clock = PARSE_PATTERNS[:clock].match(matched[1])
      next if clock.nil?

      hour = clock[1].to_i
      unless clock[5].nil?
        hour %= 12
        hour += 12 if clock[5].casecmp?("p")
      end
      found[:hour] = hour
      found[:min] = clock[2].to_i unless clock[2].nil?
      found[:sec] = clock[3].to_i unless clock[3].nil?
      found[:sec_fraction] = Rational(clock[4].to_i, 10**clock[4].length) unless clock[4].nil?
    end
  end
  private_class_method :parse_time

  def self.parse_era_bc?(era)
    !era.nil? && era.start_with?("B", "b")
  end
  private_class_method :parse_era_bc?

  def self.parse_month_number(word)
    PARSE_MONTHS.index(word[0, 3].downcase) + 1
  end
  private_class_method :parse_month_number

  def self.parse_eu(text, found)
    parse_sub(text, :eu) do |matched|
      parse_s3e(found, matched[4], parse_month_number(matched[2]), matched[1],
                parse_era_bc?(matched[3]))
    end
  end
  private_class_method :parse_eu

  def self.parse_us(text, found)
    parse_sub(text, :us) do |matched|
      parse_s3e(found, matched[4], parse_month_number(matched[1]), matched[2],
                parse_era_bc?(matched[3]))
    end
  end
  private_class_method :parse_us

  def self.parse_iso(text, found)
    parse_sub(text, :iso) do |matched|
      parse_s3e(found, matched[1], matched[2], matched[3], false)
    end
  end
  private_class_method :parse_iso

  def self.parse_iso2(text, found)
    read = parse_sub(text, :iso21) do |matched|
      found[:cwyear] = matched[1].to_i unless matched[1].nil?
      found[:cweek] = matched[2].to_i
      found[:cwday] = matched[3].to_i unless matched[3].nil?
    end
    return true if read
    read = parse_sub(text, :iso22) { |matched| found[:cwday] = matched[1].to_i }
    return true if read
    read = parse_sub(text, :iso23) do |matched|
      found[:mon] = matched[1].to_i unless matched[1].nil?
      found[:mday] = matched[2].to_i
    end
    return true if read
    read = parse_sub(text, :iso24) do |matched|
      found[:mon] = matched[1].to_i
      found[:mday] = matched[2].to_i unless matched[2].nil?
    end
    return true if read
    unless PARSE_PATTERNS[:iso25_guard].match?(text)
      read = parse_sub(text, :iso25) do |matched|
        found[:year] = matched[1].to_i
        found[:yday] = matched[2].to_i
      end
      return true if read
    end
    return false if PARSE_PATTERNS[:iso26_guard].match?(text)
    parse_sub(text, :iso26) { |matched| found[:yday] = matched[1].to_i }
  end
  private_class_method :parse_iso2

  def self.parse_jis(text, found)
    parse_sub(text, :jis) do |matched|
      found[:year] = matched[2].to_i + JAPANESE_ERA_OFFSETS[matched[1].downcase]
      found[:mon] = matched[3].to_i
      found[:mday] = matched[4].to_i
    end
  end
  private_class_method :parse_jis

  def self.parse_vms(text, found)
    read = parse_sub(text, :vms11) do |matched|
      parse_s3e(found, matched[3], parse_month_number(matched[2]), matched[1], false)
    end
    return true if read
    parse_sub(text, :vms12) do |matched|
      parse_s3e(found, matched[3], parse_month_number(matched[1]), matched[2], false)
    end
  end
  private_class_method :parse_vms

  def self.parse_sla(text, found)
    parse_sub(text, :sla) do |matched|
      parse_s3e(found, matched[1], matched[2], matched[3], false)
    end
  end
  private_class_method :parse_sla

  def self.parse_dot(text, found)
    parse_sub(text, :dot) do |matched|
      parse_s3e(found, matched[1], matched[2], matched[3], false)
    end
  end
  private_class_method :parse_dot

  def self.parse_year(text, found)
    parse_sub(text, :year) { |matched| found[:year] = matched[1].to_i }
  end
  private_class_method :parse_year

  def self.parse_mon(text, found)
    parse_sub(text, :mon) { |matched| found[:mon] = parse_month_number(matched[1]) }
  end
  private_class_method :parse_mon

  def self.parse_mday(text, found)
    parse_sub(text, :mday) { |matched| found[:mday] = matched[1].to_i }
  end
  private_class_method :parse_mday

  # A run of digits read by its length as a date, a time, or both, with
  # the clock digits and zone that may follow it.
  def self.parse_ddd(text, found)
    parse_sub(text, :ddd) do |matched|
      digits = matched[2]
      clock = matched[3]
      fraction = matched[4]
      zone = matched[5]
      negative = matched[1] == "-"
      pick = ->(from, width) { digits[from, width].to_i }
      signed = ->(value) { negative ? -value : value }
      time_only = clock.nil? && !fraction.nil?
      size = digits.length
      case size
      when 2
        if time_only
          found[:sec] = pick.(size - 2, 2)
        else
          found[:mday] = pick.(0, 2)
        end
      when 4
        if time_only
          found[:sec] = pick.(size - 2, 2)
          found[:min] = pick.(size - 4, 2)
        else
          found[:mon] = pick.(0, 2)
          found[:mday] = pick.(2, 2)
        end
      when 6
        if time_only
          found[:sec] = pick.(size - 2, 2)
          found[:min] = pick.(size - 4, 2)
          found[:hour] = pick.(size - 6, 2)
        else
          found[:year] = signed.(pick.(0, 2))
          found[:mon] = pick.(2, 2)
          found[:mday] = pick.(4, 2)
        end
      when 8, 10, 12, 14
        if time_only
          found[:sec] = pick.(size - 2, 2)
          found[:min] = pick.(size - 4, 2)
          found[:hour] = pick.(size - 6, 2)
          found[:mday] = pick.(size - 8, 2)
          found[:mon] = pick.(size - 10, 2) if size >= 10
          found[:year] = signed.(pick.(size - 12, 2)) if size == 12
          if size == 14
            found[:year] = signed.(pick.(size - 14, 4))
            found[:_comp] = false
          end
        else
          found[:year] = signed.(pick.(0, 4))
          found[:mon] = pick.(4, 2)
          found[:mday] = pick.(6, 2)
          found[:hour] = pick.(8, 2) if size >= 10
          found[:min] = pick.(10, 2) if size >= 12
          found[:sec] = pick.(12, 2) if size >= 14
          found[:_comp] = false
        end
      when 3
        if time_only
          found[:sec] = pick.(size - 2, 2)
          found[:min] = pick.(size - 3, 1)
        else
          found[:yday] = pick.(0, 3)
        end
      when 5
        if time_only
          found[:sec] = pick.(size - 2, 2)
          found[:min] = pick.(size - 4, 2)
          found[:hour] = pick.(size - 5, 1)
        else
          found[:year] = signed.(pick.(0, 2))
          found[:yday] = pick.(2, 3)
        end
      when 7
        if time_only
          found[:sec] = pick.(size - 2, 2)
          found[:min] = pick.(size - 4, 2)
          found[:hour] = pick.(size - 6, 2)
          found[:mday] = pick.(size - 7, 1)
        else
          found[:year] = signed.(pick.(0, 4))
          found[:yday] = pick.(4, 3)
        end
      end
      if !clock.nil? && [2, 4, 6].include?(clock.length)
        if fraction.nil?
          found[:hour] = clock[0, 2].to_i
          found[:min] = clock[2, 2].to_i if clock.length >= 4
          found[:sec] = clock[4, 2].to_i if clock.length >= 6
        else
          found[:sec] = clock[-2, 2].to_i
          found[:min] = clock[-4, 2].to_i if clock.length >= 4
          found[:hour] = clock[-6, 2].to_i if clock.length >= 6
        end
      end
      found[:sec_fraction] = Rational(fraction.to_i, 10**fraction.length) unless fraction.nil?
      unless zone.nil?
        found[:zone] = zone
        if zone.start_with?("[")
          inner = zone[1, zone.length - 2]
          colon = inner.index(":")
          if colon.nil?
            found[:zone] = inner
            found[:offset] = zone_to_diff(digit?(inner[0].to_s) ? "+" + inner : inner)
          else
            found[:zone] = inner[(colon + 1)..]
            found[:offset] = zone_to_diff(inner[0, colon + 1])
          end
        end
      end
    end
  end
  private_class_method :parse_ddd

  def self.parse_bc(text, found)
    parse_sub(text, :bc) { found[:_bc] = true }
  end
  private_class_method :parse_bc

  def self.parse_frag(text, found)
    parse_sub(text, :frag) do |matched|
      number = matched[1].to_i
      if !found[:hour].nil? && found[:mday].nil? && number >= 1 && number <= 31
        found[:mday] = number
      end
      if !found[:mday].nil? && found[:hour].nil? && number >= 0 && number <= 24
        found[:hour] = number
      end
    end
  end
  private_class_method :parse_frag

  # Where the first sign or digit in a piece of text sits, and the span
  # of the number that starts there, or nil when it holds neither.
  def self.parse_number_span(piece, signed)
    start = piece.index(signed ? /[-+\d]/ : /\d/)
    return nil if start.nil?
    digits_at = start
    digits_at += 1 if signed && (piece[start] == "-" || piece[start] == "+")
    finish = digits_at
    finish += 1 while finish < piece.length && digit?(piece[finish])
    [start, digits_at, finish]
  end
  private_class_method :parse_number_span

  # Three pieces read as a year, a month and a day, swapped about by their
  # lengths and quotes the way Ruby settles which is which.
  def self.parse_s3e(found, year, month, day, bc)
    month = month.to_s
    year, month, day = day, year, month if !year.nil? && day.nil?
    if year.nil?
      if !day.nil? && day.length > 2
        year = day
        day = nil
      end
      if !day.nil? && day.start_with?("'")
        year = day
        day = nil
      end
    end
    unless year.nil?
      span = parse_number_span(year, true)
      if !span.nil? && span[2] < year.length
        start, _, finish = span
        held = year
        year = day
        day = held[start...finish]
      end
    end
    if !month.nil? && (month.start_with?("'") || month.length > 2)
      year, month, day = month, day, year
    end
    if !day.nil? && (day.start_with?("'") || day.length > 2)
      year, day = day, year
    end

    completes = nil
    unless year.nil?
      span = parse_number_span(year, true)
      unless span.nil?
        start, digits_at, finish = span
        completes = false if digits_at > start || finish - digits_at > 2
        found[:year] = year[start...finish].to_i
      end
    end
    found[:_bc] = true if bc
    unless month.nil?
      span = parse_number_span(month, false)
      found[:mon] = month[span[0]...span[2]].to_i unless span.nil?
    end
    unless day.nil?
      span = parse_number_span(day, false)
      found[:mday] = day[span[0]...span[2]].to_i unless span.nil?
    end
    found[:_comp] = completes unless completes.nil?
  end
  private_class_method :parse_s3e

  def self.digit?(character)
    character >= "0" && character <= "9"
  end

  # A year written with two digits, moved into the window 1969 through 2068
  # the way Ruby completes one.
  def self.complete_year(value, comp)
    return value unless comp
    return value if value >= 100 || value < 0
    value < 69 ? 2000 + value : 1900 + value
  end

  def self.coerce_to_string(text)
    return text.to_s if text.is_a?(String)
    if text.respond_to?(:to_str)
      converted = text.to_str
      return converted.to_s if converted.is_a?(String)
    end
    raise TypeError, "no implicit conversion of #{text.class} into String"
  end

  def self.parse(text = "-4712-01-01", comp = true, start = ITALY)
    new_by_frags(_parse(text, comp), start)
  end

  # A date read out of text by a strftime template, returned as the fields
  # the template named, or nil when the text does not match.
  def self._strptime(text, format = "%F")
    read_strptime(text, format)
  end

  def self.read_strptime(text, format)
    text = coerce_to_string(text)
    format = coerce_to_string(format)
    found = {}
    at = strptime_reading(text, 0, format, found)
    found[:leftover] = text[at..] if !at.nil? && at < text.length
    return nil if found.key?(:_fail)

    century = found.delete(:_cent)
    unless century.nil?
      found[:cwyear] += century * 100 unless found[:cwyear].nil?
      found[:year] += century * 100 unless found[:year].nil?
    end
    meridian = found.delete(:_merid)
    if !meridian.nil? && !found[:hour].nil?
      found[:hour] = found[:hour] % 12 + meridian
    end
    found
  end
  private_class_method :read_strptime

  DAY_NAMES_READ = %w[Sunday Monday Tuesday Wednesday Thursday Friday Saturday].freeze
  MONTH_NAMES_READ = %w[January February March April May June July August
                        September October November December].freeze
  # The directives that read a number, which shorten one before them that
  # reads a number of no fixed width.
  NUMBER_DIRECTIVES = "CDdeFGgHIjkLlMmNQRrSsTUuVvWwXxYy"
  STRPTIME_ZONE = /\A((?:gmt|utc?)?[-+]\d+(?:[,.:]\d+(?::\d+)?)?|(?-i:[[:alpha:].\s]+)(?:standard|daylight)\s+time\b|(?-i:[[:alpha:]]+)(?:\s+dst)?\b)/i
  private_constant :DAY_NAMES_READ, :MONTH_NAMES_READ, :NUMBER_DIRECTIVES, :STRPTIME_ZONE

  # Reads `text` from `at` against `format`, recording what each directive
  # names in `found`, and answers where reading stopped. A mismatch records
  # :_fail and answers nil.
  def self.strptime_reading(text, at, format, found)
    index = 0
    while index < format.length
      if strptime_space?(format[index])
        at += 1 while at < text.length && strptime_space?(text[at])
        index += 1
        index += 1 while index < format.length && strptime_space?(format[index])
        next
      end
      return strptime_fail(found) if at >= text.length

      if format[index] != "%"
        return strptime_fail(found) unless text[at] == format[index]
        at += 1
        index += 1
        next
      end

      index += 1
      directive = format[index]
      if directive == "E" || directive == "O"
        allowed = directive == "E" ? "cCxXyY" : "deHImMSuUVwWy"
        following = format[index + 1]
        if !following.nil? && allowed.include?(following)
          index += 1
          directive = following
        else
          # A modifier before a directive it does not apply to is read as
          # the text of the template.
          return strptime_fail(found) unless text[at] == "%"
          at += 1
          next
        end
      elsif directive == ":"
        colons = 1
        colons += 1 while colons < 3 && format[index + colons] == ":"
        return strptime_fail(found) unless format[index + colons] == "z"
        index += colons
        directive = "z"
      end
      at = strptime_directive(directive, text, at, format, index, found)
      return nil if at.nil?
      index += 1
    end
    at
  end
  private_class_method :strptime_reading

  # What one directive reads, recorded in `found`, answering where reading
  # stopped, or nil when the text does not match.
  def self.strptime_directive(directive, text, at, format, index, found)
    number_follows = strptime_number_pattern?(format, index + 1)
    case directive
    when "A", "a"
      strptime_name(found, :wday, DAY_NAMES_READ, 0, text, at)
    when "B", "b", "h"
      strptime_name(found, :mon, MONTH_NAMES_READ, 1, text, at)
    when "C"
      strptime_number(found, :_cent, text, at, number_follows ? 2 : nil)
    when "c" then strptime_nested(text, at, "%a %b %e %H:%M:%S %Y", found)
    when "D", "x" then strptime_nested(text, at, "%m/%d/%y", found)
    when "d", "e"
      strptime_padded(found, :mday, text, at, 1, 31)
    when "F" then strptime_nested(text, at, "%Y-%m-%d", found)
    when "G"
      strptime_number(found, :cwyear, text, at, number_follows ? 4 : nil)
    when "g"
      at = strptime_ranged(found, :cwyear, text, at, 2, 0, 99)
      found[:_cent] = found[:cwyear] >= 69 ? 19 : 20 if !at.nil? && found[:_cent].nil?
      at
    when "H", "k"
      strptime_padded(found, :hour, text, at, 0, 24)
    when "I", "l"
      strptime_padded(found, :hour, text, at, 1, 12)
    when "j"
      strptime_ranged(found, :yday, text, at, 3, 1, 366)
    when "L", "N"
      sign = 1
      if text[at] == "-" || text[at] == "+"
        sign = -1 if text[at] == "-"
        at += 1
      end
      start = at
      digits = read_strptime_digits(text, at, number_follows ? (directive == "L" ? 3 : 9) : nil)
      return strptime_fail(found) if digits.nil?
      at += digits.length
      found[:sec_fraction] = Rational(sign * digits.to_i, 10**(at - start))
      at
    when "M"
      strptime_ranged(found, :min, text, at, 2, 0, 59)
    when "m"
      strptime_ranged(found, :mon, text, at, 2, 1, 12)
    when "n", "t" then strptime_nested(text, at, " ", found)
    when "P", "p"
      strptime_meridian(found, text, at)
    when "Q"
      sign = 1
      if text[at] == "-"
        sign = -1
        at += 1
      end
      digits = read_strptime_digits(text, at, nil)
      return strptime_fail(found) if digits.nil?
      found[:seconds] = Rational(sign * digits.to_i, 1000)
      at + digits.length
    when "R" then strptime_nested(text, at, "%H:%M", found)
    when "r" then strptime_nested(text, at, "%I:%M:%S %p", found)
    when "S"
      strptime_ranged(found, :sec, text, at, 2, 0, 60)
    when "s"
      sign = 1
      if text[at] == "-"
        sign = -1
        at += 1
      end
      digits = read_strptime_digits(text, at, nil)
      return strptime_fail(found) if digits.nil?
      found[:seconds] = sign * digits.to_i
      at + digits.length
    when "T", "X" then strptime_nested(text, at, "%H:%M:%S", found)
    when "U", "W"
      strptime_ranged(found, directive == "U" ? :wnum0 : :wnum1, text, at, 2, 0, 53)
    when "u"
      strptime_ranged(found, :cwday, text, at, 1, 1, 7)
    when "V"
      strptime_ranged(found, :cweek, text, at, 2, 1, 53)
    when "v" then strptime_nested(text, at, "%e-%b-%Y", found)
    when "w"
      strptime_ranged(found, :wday, text, at, 1, 0, 6)
    when "Y"
      sign = 1
      if text[at] == "-" || text[at] == "+"
        sign = -1 if text[at] == "-"
        at += 1
      end
      digits = read_strptime_digits(text, at, number_follows ? 4 : nil)
      return strptime_fail(found) if digits.nil?
      found[:year] = sign * digits.to_i
      at + digits.length
    when "y"
      at = strptime_ranged(found, :year, text, at, 2, 0, 99)
      found[:_cent] = found[:year] >= 69 ? 19 : 20 if !at.nil? && found[:_cent].nil?
      at
    when "Z", "z"
      matched = STRPTIME_ZONE.match(text[at..])
      return strptime_fail(found) if matched.nil?
      found[:zone] = matched[1]
      found[:offset] = zone_to_diff(matched[1])
      at + matched.end(0)
    when "%"
      return strptime_fail(found) unless text[at] == "%"
      at + 1
    when "+" then strptime_nested(text, at, "%a %b %e %H:%M:%S %Z %Y", found)
    else
      # An unknown directive, or a `%` that ends the template, is read as
      # the text it is written with.
      return strptime_fail(found) unless text[at] == "%"
      at += 1
      unless directive.nil?
        return strptime_fail(found) unless text[at] == directive
        at += 1
      end
      at
    end
  end
  private_class_method :strptime_directive

  def self.strptime_fail(found)
    found[:_fail] = true
    nil
  end
  private_class_method :strptime_fail

  # Whether the template at `index` goes on with digits or a directive that
  # reads a number.
  def self.strptime_number_pattern?(format, index)
    character = format[index]
    return false if character.nil?
    return true if digit?(character)
    return false unless character == "%"

    index += 1
    index += 1 if format[index] == "E" || format[index] == "O"
    character = format[index]
    !character.nil? && (NUMBER_DIRECTIVES.include?(character) || digit?(character))
  end
  private_class_method :strptime_number_pattern?

  def self.strptime_space?(character)
    " \t\n\v\f\r".include?(character)
  end
  private_class_method :strptime_space?

  # The digits at `at`, at most `width` of them when a width is given, or
  # nil when there are none.
  def self.read_strptime_digits(text, at, width)
    finish = at
    while finish < text.length && digit?(text[finish])
      finish += 1
      break if !width.nil? && finish - at == width
    end
    finish == at ? nil : text[at...finish]
  end
  private_class_method :read_strptime_digits

  def self.strptime_number(found, field, text, at, width)
    digits = read_strptime_digits(text, at, width)
    return strptime_fail(found) if digits.nil?
    found[field] = digits.to_i
    at + digits.length
  end
  private_class_method :strptime_number

  def self.strptime_ranged(found, field, text, at, width, low, high)
    digits = read_strptime_digits(text, at, width)
    return strptime_fail(found) if digits.nil?
    value = digits.to_i
    return strptime_fail(found) if value < low || value > high
    found[field] = value
    at + digits.length
  end
  private_class_method :strptime_ranged

  # A number of one or two digits, where a space may stand for the first.
  def self.strptime_padded(found, field, text, at, low, high)
    return strptime_ranged(found, field, text, at + 1, 1, low, high) if text[at] == " "
    strptime_ranged(found, field, text, at, 2, low, high)
  end
  private_class_method :strptime_padded

  def self.strptime_name(found, field, names, first, text, at)
    names.each_with_index do |name, number|
      [name.length, 3].each do |length|
        written = text[at, length]
        next unless !written.nil? && written.length == length &&
                    written.casecmp?(name[0, length])
        found[field] = number + first
        return at + length
      end
    end
    strptime_fail(found)
  end
  private_class_method :strptime_name

  def self.strptime_meridian(found, text, at)
    return strptime_fail(found) if text.length - at < 2
    character = text[at]
    hour = character == "P" || character == "p" ? 12 : 0
    return strptime_fail(found) if hour == 0 && character != "A" && character != "a"
    character = text[at + 1]
    if character == "."
      return strptime_fail(found) if text.length - at < 4 || text[at + 3] != "."
      at += 2
      character = text[at]
    end
    return strptime_fail(found) unless character == "M" || character == "m"
    found[:_merid] = hour
    at + 2
  end
  private_class_method :strptime_meridian

  # A directive that stands for a template of others, read against what is
  # left of the text.
  def self.strptime_nested(text, at, format, found)
    read = strptime_reading(text[at..], 0, format, found)
    read.nil? ? nil : at + read
  end
  private_class_method :strptime_nested

  def self.strptime(text = "-4712-01-01", format = "%F", start = ITALY)
    new_by_frags(_strptime(text, format), start)
  end

  # The date a set of fields read from text names, filling what the text
  # left out from today.
  def self.new_by_frags(fields, start)
    raise Date::Error, "invalid date" if fields.nil?
    if civil_fields?(fields)
      jd = valid_civil_jd(fields[:year], fields[:mon], fields[:mday], start)
    else
      fields = complete_frags(rewrite_frags(fields))
      jd = valid_date_frags(fields, start)
    end
    raise Date::Error, "invalid date" if jd.nil?
    from_jd(jd, start)
  end
  private_class_method :new_by_frags

  def self.civil_fields?(fields)
    fields[:jd].nil? && fields[:yday].nil? &&
      !fields[:year].nil? && !fields[:mon].nil? && !fields[:mday].nil?
  end
  private_class_method :civil_fields?

  # Seconds since the epoch, written as the day and the time of day they
  # name.
  def self.rewrite_frags(fields)
    fields = fields.dup
    seconds = fields.delete(:seconds)
    return fields if seconds.nil?

    seconds += fields[:offset] unless fields[:offset].nil?
    rest = seconds % 86400
    fields[:jd] = 2_440_588 + seconds.div(86400)
    fields[:hour] = rest.div(3600)
    rest %= 3600
    fields[:min] = rest.div(60)
    rest %= 60
    fields[:sec] = rest.div(1)
    fields[:sec_fraction] = rest % 1
    fields
  end
  private_class_method :rewrite_frags

  # The ways of naming a day, each with the fields it reads from.
  FRAG_KINDS = [
    [:time, %i[hour min sec]],
    [nil, %i[jd]],
    [:ordinal, %i[year yday hour min sec]],
    [:civil, %i[year mon mday hour min sec]],
    [:commercial, %i[cwyear cweek cwday hour min sec]],
    [:wday, %i[wday hour min sec]],
    [:wnum0, %i[year wnum0 wday hour min sec]],
    [:wnum1, %i[year wnum1 wday hour min sec]],
    [nil, %i[cwyear cweek wday hour min sec]],
    [nil, %i[year wnum0 cwday hour min sec]],
    [nil, %i[year wnum1 cwday hour min sec]]
  ].freeze
  private_constant :FRAG_KINDS

  # The fields with the way of naming a day they come nearest to filled
  # in from today, and the time of day filled with zeros.
  def self.complete_frags(fields)
    kind = nil
    names = nil
    most = 0
    FRAG_KINDS.each do |candidate, candidate_names|
      count = candidate_names.count { |name| !fields[name].nil? }
      next unless count > most
      most = count
      kind = candidate
      names = candidate_names
    end
    today = nil
    if !kind.nil? && names.length > most
      case kind
      when :ordinal
        fields[:year] = (today ||= current_day(ITALY)).year if fields[:year].nil?
        fields[:yday] = 1 if fields[:yday].nil?
      when :civil, :commercial, :wnum0, :wnum1
        names.each do |name|
          break unless fields[name].nil?
          fields[name] = (today ||= current_day(ITALY)).send(name)
        end
        defaults = { civil: { mon: 1, mday: 1 }, commercial: { cweek: 1, cwday: 1 },
                     wnum0: { wnum0: 0, wday: 0 }, wnum1: { wnum1: 0, wday: 1 } }
        defaults[kind].each { |name, value| fields[name] = value if fields[name].nil? }
      when :wday
        today ||= current_day(ITALY)
        fields[:jd] = (today - today.wday + fields[:wday]).jd
      end
    end
    if kind == :time && self <= DateTime && fields[:jd].nil?
      fields[:jd] = (today || current_day(ITALY)).jd
    end
    fields[:hour] = 0 if fields[:hour].nil?
    fields[:min] = 0 if fields[:min].nil?
    if fields[:sec].nil?
      fields[:sec] = 0
    elsif fields[:sec] > 59
      fields[:sec] = 59
    end
    fields
  end
  private_class_method :complete_frags

  # The Julian Day Number the first way of naming a day the fields fill in
  # stands for, or nil when none of them names one.
  def self.valid_date_frags(fields, start)
    return fields[:jd] unless fields[:jd].nil?

    if !fields[:yday].nil? && !fields[:year].nil?
      jd = valid_ordinal_jd(fields[:year], fields[:yday], start)
      return jd unless jd.nil?
    end
    if !fields[:mday].nil? && !fields[:mon].nil? && !fields[:year].nil?
      jd = valid_civil_jd(fields[:year], fields[:mon], fields[:mday], start)
      return jd unless jd.nil?
    end

    weekday = fields[:cwday]
    weekday = fields[:wday] == 0 ? 7 : fields[:wday] if weekday.nil? && !fields[:wday].nil?
    if !weekday.nil? && !fields[:cweek].nil? && !fields[:cwyear].nil? &&
       valid_commercial?(fields[:cwyear], fields[:cweek], weekday, start)
      return Date.commercial(fields[:cwyear], fields[:cweek], weekday, start).jd
    end

    weekday = fields[:wday]
    weekday = fields[:cwday] == 7 ? 0 : fields[:cwday] if weekday.nil? && !fields[:cwday].nil?
    if !weekday.nil? && !fields[:wnum0].nil? && !fields[:year].nil?
      jd = valid_weeknum_jd(fields[:year], fields[:wnum0], weekday, 0, start)
      return jd unless jd.nil?
    end

    weekday = fields[:wday] || fields[:cwday]
    weekday = (weekday - 1) % 7 unless weekday.nil?
    if !weekday.nil? && !fields[:wnum1].nil? && !fields[:year].nil?
      jd = valid_weeknum_jd(fields[:year], fields[:wnum1], weekday, 1, start)
      return jd unless jd.nil?
    end
    nil
  end
  private_class_method :valid_date_frags

  # The day a week number names in a year, with weeks opening on Sunday
  # when `first` is 0 and on Monday when it is 1, or nil when that week and
  # day do not fall in that year.
  def self.valid_weeknum_jd(year, week, day, first, start)
    day += 7 if day < 0
    if week < 0
      following = weeknum_to_jd(year + 1, 1, first, first, start)
      held_year, week, = jd_to_weeknum(following + week * 7, first, start)
      return nil unless held_year == year
    end
    jd = weeknum_to_jd(year, week, day, first, start)
    held = jd_to_weeknum(jd, first, start)
    held == [year, week, day] ? jd : nil
  end
  private_class_method :valid_weeknum_jd

  def self.weeknum_to_jd(year, week, day, first, start)
    opening = first_day_of_year(year, start) + 6
    (opening - (opening - first + 1) % 7 - 7) + 7 * week + day
  end
  private_class_method :weeknum_to_jd

  def self.jd_to_weeknum(jd, first, start)
    year = Date.jd(jd, start).year
    opening = first_day_of_year(year, start) + 6
    counted = jd - (opening - (opening - first + 1) % 7) + 7
    [year, counted.div(7), counted % 7]
  end
  private_class_method :jd_to_weeknum

  # The first day a year holds, which a change of calendar can move past
  # the first of January.
  def self.first_day_of_year(year, start)
    (1..30).each do |day|
      found = valid_civil_jd(year, 1, day, start)
      return found unless found.nil?
    end
    nil
  end
  private_class_method :first_day_of_year

  # Zones by name, as MRI's date library reads them, in seconds from UTC.
  ZONE_OFFSETS = {
    "ut" => 0, "gmt" => 0, "est" => -18000, "edt" => -14400,
    "cst" => -21600, "cdt" => -18000, "mst" => -25200, "mdt" => -21600,
    "pst" => -28800, "pdt" => -25200, "a" => 3600, "b" => 7200,
    "c" => 10800, "d" => 14400, "e" => 18000, "f" => 21600,
    "g" => 25200, "h" => 28800, "i" => 32400, "k" => 36000,
    "l" => 39600, "m" => 43200, "n" => -3600, "o" => -7200,
    "p" => -10800, "q" => -14400, "r" => -18000, "s" => -21600,
    "t" => -25200, "u" => -28800, "v" => -32400, "w" => -36000,
    "x" => -39600, "y" => -43200, "z" => 0, "utc" => 0,
    "wet" => 0, "at" => -7200, "brst" => -7200, "ndt" => -5400,
    "art" => -10800, "adt" => -10800, "brt" => -10800, "clst" => -10800,
    "nst" => -9000, "ast" => -14400, "clt" => -14400, "akdt" => -28800,
    "ydt" => -28800, "akst" => -32400, "hadt" => -32400, "hdt" => -32400,
    "yst" => -32400, "ahst" => -36000, "cat" => 7200, "hast" => -36000,
    "hst" => -36000, "nt" => -39600, "idlw" => -43200, "bst" => 3600,
    "cet" => 3600, "fwt" => 3600, "met" => 3600, "mewt" => 3600,
    "mez" => 3600, "swt" => 3600, "wat" => 3600, "west" => 3600,
    "cest" => 7200, "eet" => 7200, "fst" => 7200, "mest" => 7200,
    "mesz" => 7200, "sast" => 7200, "sst" => -39600, "bt" => 10800,
    "eat" => 10800, "eest" => 10800, "msk" => 10800, "msd" => 14400,
    "zp4" => 14400, "zp5" => 18000, "ist" => 19800, "zp6" => 21600,
    "wast" => 7200, "cct" => 23400, "sgt" => 28800, "wadt" => 28800,
    "jst" => 32400, "kst" => 32400, "east" => -21600, "gst" => 36000,
    "eadt" => 39600, "idle" => 43200, "nzst" => 43200, "nzt" => 43200,
    "nzdt" => 46800, "afghanistan" => 16200, "alaskan" => -32400, "arab" => 10800,
    "arabian" => 14400, "arabic" => 10800, "atlantic" => -14400, "aus central" => 34200,
    "aus eastern" => 36000, "azores" => -3600, "canada central" => -21600, "cape verde" => -3600,
    "caucasus" => 14400, "cen. australia" => 34200, "central america" => -21600, "central asia" => 21600,
    "central europe" => 3600, "central european" => 3600, "central pacific" => 39600, "central" => -21600,
    "china" => 28800, "dateline" => -43200, "e. africa" => 10800, "e. australia" => 36000,
    "e. europe" => 7200, "e. south america" => -10800, "eastern" => -18000, "egypt" => 7200,
    "ekaterinburg" => 18000, "fiji" => 43200, "fle" => 7200, "greenland" => -10800,
    "greenwich" => 0, "gtb" => 7200, "hawaiian" => -36000, "india" => 19800,
    "iran" => 12600, "jerusalem" => 7200, "korea" => 32400, "mexico" => -21600,
    "mid-atlantic" => -7200, "mountain" => -25200, "myanmar" => 23400, "n. central asia" => 21600,
    "nepal" => 20700, "new zealand" => 43200, "newfoundland" => -12600, "north asia east" => 28800,
    "north asia" => 25200, "pacific sa" => -14400, "pacific" => -28800, "romance" => 3600,
    "russian" => 10800, "sa eastern" => -10800, "sa pacific" => -18000, "sa western" => -14400,
    "samoa" => -39600, "se asia" => 25200, "malay peninsula" => 28800, "south africa" => 7200,
    "sri lanka" => 21600, "taipei" => 28800, "tasmania" => 36000, "tokyo" => 32400,
    "tonga" => 46800, "us eastern" => -18000, "us mountain" => -25200, "vladivostok" => 36000,
    "w. australia" => 28800, "w. central africa" => 3600, "w. europe" => 3600, "west asia" => 18000,
    "west pacific" => 36000, "yakutsk" => 32400, "acdt" => 37800, "acst" => 34200,
    "act" => -18000, "acwst" => 31500, "aedt" => 39600, "aest" => 36000,
    "aft" => 16200, "almt" => 21600, "anast" => 43200, "anat" => 43200,
    "aoe" => -43200, "aqtt" => 18000, "awdt" => 32400, "awst" => 28800,
    "azost" => 0, "azot" => -3600, "azst" => 18000, "azt" => 14400,
    "bnt" => 28800, "bot" => -14400, "btt" => 21600, "cast" => 28800,
    "chadt" => 49500, "chast" => 45900, "chost" => 32400, "chot" => 28800,
    "chst" => 36000, "chut" => 36000, "cidst" => -14400, "cist" => -18000,
    "ckt" => -36000, "cot" => -18000, "cvt" => -3600, "cxt" => 25200,
    "davt" => 25200, "ddut" => 36000, "easst" => -18000, "ect" => -18000,
    "egst" => 0, "egt" => -3600, "fet" => 10800, "fjst" => 46800,
    "fjt" => 43200, "fkst" => -10800, "fkt" => -14400, "fnt" => -7200,
    "galt" => -21600, "gamt" => -32400, "get" => 14400, "gft" => -10800,
    "gilt" => 43200, "gyt" => -14400, "hkt" => 28800, "hovst" => 28800,
    "hovt" => 25200, "ict" => 25200, "idt" => 10800, "iot" => 21600,
    "irdt" => 16200, "irkst" => 32400, "irkt" => 28800, "irst" => 12600,
    "kgt" => 21600, "kost" => 39600, "krast" => 28800, "krat" => 25200,
    "kuyt" => 14400, "lhdt" => 39600, "lhst" => 37800, "lint" => 50400,
    "magst" => 43200, "magt" => 39600, "mart" => -30600, "mawt" => 18000,
    "mht" => 43200, "mmt" => 23400, "mut" => 14400, "mvt" => 18000,
    "myt" => 28800, "nct" => 39600, "nfdt" => 43200, "nft" => 39600,
    "novst" => 25200, "novt" => 25200, "npt" => 20700, "nrt" => 43200,
    "nut" => -39600, "omsst" => 25200, "omst" => 21600, "orat" => 18000,
    "pet" => -18000, "petst" => 43200, "pett" => 43200, "pgt" => 36000,
    "phot" => 46800, "pht" => 28800, "pkt" => 18000, "pmdt" => -7200,
    "pmst" => -10800, "pont" => 39600, "pwt" => 32400, "pyst" => -10800,
    "qyzt" => 21600, "ret" => 14400, "rott" => -10800, "sakt" => 39600,
    "samt" => 14400, "sbt" => 39600, "sct" => 14400, "sret" => 39600,
    "srt" => -10800, "syot" => 10800, "taht" => -36000, "tft" => 18000,
    "tjt" => 18000, "tkt" => 46800, "tlt" => 32400, "tmt" => 18000,
    "tost" => 50400, "tot" => 46800, "trt" => 10800, "tvt" => 43200,
    "ulast" => 32400, "ulat" => 28800, "uyst" => -7200, "uyt" => -10800,
    "uzt" => 18000, "vet" => -14400, "vlast" => 39600, "vlat" => 36000,
    "vost" => 21600, "vut" => 39600, "wakt" => 43200, "warst" => -10800,
    "wft" => 43200, "wgst" => -3600, "wgt" => -7200, "wib" => 25200,
    "wit" => 32400, "wita" => 28800, "wt" => 0, "yakst" => 36000,
    "yakt" => 32400, "yapt" => 36000, "yekst" => 21600, "yekt" => 18000,
  }.freeze
  private_constant :ZONE_OFFSETS

  # The offset from UTC in seconds a zone written as text names: a zone's
  # name, with "standard time", "daylight time" or "dst" after it, or a
  # signed offset, with GMT or UTC before it. Nil when it names none.
  def self.zone_to_diff(text)
    text = text.to_s
    length = text.length
    daylight = false
    if (taken = zone_word_at_end(text, length, "time")) > 0
      length -= taken
      if (taken_more = zone_word_at_end(text, length, "standard")) > 0
        length -= taken_more
      elsif (taken_more = zone_word_at_end(text, length, "daylight")) > 0
        length -= taken_more
        daylight = true
      else
        length += taken
      end
    elsif (taken = zone_word_at_end(text, length, "dst")) > 0
      length -= taken
      daylight = true
    end

    name = text[0, length].gsub(/[ \t\n\v\f\r]+/, " ").sub(/ \z/, "").downcase
    named = ZONE_OFFSETS[name]
    return daylight ? named + 3600 : named unless named.nil?

    written = text[0, length]
    written = written[3..] if written.length > 3 && %w[gmt utc].include?(written[0, 3].downcase)
    return nil unless written.start_with?("+", "-")

    negative = written.start_with?("-")
    written = written[1..]
    hour = written[/\A\d*/].to_i
    after = written[/\A\d*/].length
    minute = 0
    second = 0
    if written[after] == ":"
      return nil if hour > 23
      rest = written[(after + 1)..]
      minute = rest[/\A\d*/].to_i
      return nil if minute > 59
      rest = rest[rest[/\A\d*/].length..]
      if rest.start_with?(":")
        second = rest[1..][/\A\d*/].to_i
        return nil if second > 59
      end
    elsif written[after] == "," || written[after] == "."
      return nil if hour > 23
      rest = written[(after + 1)..]
      digits = rest[0, 7][/\A\d*/]
      counted = digits.to_i
      following = rest[digits.length]
      # Round half to even on the first digit past the seventh.
      if !following.nil? && following >= (counted.odd? ? "5" : "6") && following <= "9"
        counted += 1
      end
      counted *= 36
      if negative
        hour = -hour
        counted = -counted
      end
      return hour * 3600 + counted * 10 if digits.length == 1
      return hour * 3600 + counted if digits.length <= 2
      offset = Rational(counted, 10**(digits.length - 2)) + hour * 3600
      return offset.denominator == 1 ? offset.numerator : offset
    elsif written.length > 2
      split = 2 - written.length % 2
      hour = written[0, split][/\A\d*/].to_i
      minute = written[split, 2][/\A\d*/].to_i if written.length >= 3
      second = written[split + 2, 2].to_s[/\A\d*/].to_i if written.length >= 5
    end
    seconds = hour * 3600 + minute * 60 + second
    negative ? -seconds : seconds
  end
  private_class_method :zone_to_diff

  # How much of `text` before `length` a word at its end takes, with the
  # spaces before it, or 0 when it does not end in that word after a space.
  def self.zone_word_at_end(text, length, word)
    size = word.length
    return 0 if length <= size || !" \t\n\v\f\r".include?(text[length - size - 1])
    return 0 unless text[length - size, size].casecmp?(word)
    size += 1
    size += 1 while length > size && " \t\n\v\f\r".include?(text[length - size - 1])
    size
  end
  private_class_method :zone_word_at_end

  def self.today(start = ITALY)
    now = Time.now
    civil(now.year, now.month, now.day, start)
  end

  # The day a date read without a year or a month falls back on, which a
  # program that replaces `Date.today`, as timecop does, leaves alone, since
  # MRI reads it in C.
  def self.current_day(start)
    now = Time.now
    Date.civil(now.year, now.month, now.day, start)
  end
  private_class_method :current_day

  def self.valid_civil?(year, month, day, start = ITALY)
    !valid_civil_jd(year, month, day, start).nil?
  end

  def self.valid_date?(year, month, day, start = ITALY)
    valid_civil?(year, month, day, start)
  end

  def self.valid_jd?(number, start = ITALY)
    number.is_a?(Numeric)
  end

  def self.valid_ordinal?(year, day, start = ITALY)
    !valid_ordinal_jd(year, day, start).nil?
  end

  def self.valid_ordinal_jd(year, day, start)
    return nil unless day.is_a?(Integer)
    first = month_first_jd(year, 1, start)
    last = month_first_jd(year + 1, 1, start) - 1
    found = day < 0 ? last + day + 1 : first + day - 1
    return nil if found < first || found > last
    found
  end

  # How many commercial weeks a year holds, which is 53 when it opens on a
  # Thursday and 52 otherwise, with a leap year that opens on a Wednesday
  # reaching 53 too.
  def self.commercial_weeks(year)
    opening = (gregorian_to_jd(year, 1, 1) + 1) % 7
    return 53 if opening == 4
    return 53 if opening == 3 && gregorian_leap?(year)
    52
  end

  def self.valid_commercial?(year, week, weekday, start = ITALY)
    return false unless week.is_a?(Integer) && weekday.is_a?(Integer)
    weekday = 8 + weekday if weekday < 0
    return false if weekday < 1 || weekday > 7
    weeks = commercial_weeks(year)
    week = weeks + week + 1 if week < 0
    week >= 1 && week <= weeks
  end

  def self.leap?(year)
    gregorian_leap?(year)
  end

  def self.gregorian_leap?(year)
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
  end

  def self.julian_leap?(year)
    year % 4 == 0
  end

  # The Julian Day Number a set of calendar fields stands for, or nil when
  # those fields name no day at all.
  def self.valid_civil_jd(year, month, day, start)
    return nil unless month.is_a?(Integer) && day.is_a?(Integer)
    if month < 0
      month = 13 + month
    end
    return nil if month < 1 || month > 12
    if day < 0
      last = month_last_jd(year, month, start)
      found = last + day + 1
      return nil if found < month_first_jd(year, month, start)
      return found
    end
    counted = day
    return nil if counted < 1 || counted > days_in_month(year, month, start)
    found = gregorian_to_jd(year, month, counted)
    return found if gregorian_from?(found, start)
    found = julian_to_jd(year, month, counted)
    return found unless gregorian_from?(found, start)
    nil
  end

  # The Julian Day Number the first day of a month falls on, read on
  # whichever calendar governs it.
  def self.month_first_jd(year, month, start)
    found = gregorian_to_jd(year, month, 1)
    return found if gregorian_from?(found, start)
    julian_to_jd(year, month, 1)
  end

  def self.month_last_jd(year, month, start)
    landing_year = month == 12 ? year + 1 : year
    landing_month = month == 12 ? 1 : month + 1
    month_first_jd(landing_year, landing_month, start) - 1
  end

  def self.days_in_month(year, month, start = ITALY)
    return 31 if [1, 3, 5, 7, 8, 10, 12].include?(month)
    return 30 if [4, 6, 9, 11].include?(month)
    reformed = gregorian_from?(gregorian_to_jd(year, 3, 1), start)
    leaping = reformed ? gregorian_leap?(year) : julian_leap?(year)
    leaping ? 29 : 28
  end

  # Whether a Julian Day Number falls on the Gregorian side of a calendar
  # reform. The endless calendars stand in for a reform that never happened
  # and one that always had, so they answer without comparing.
  def self.gregorian_from?(jd, start)
    return start.direction < 0 if start.is_a?(Infinity)
    jd >= start
  end

  def self.gregorian_to_jd(year, month, day)
    shift = (14 - month) / 12
    counted = year + 4800 - shift
    moved = month + 12 * shift - 3
    day + (153 * moved + 2) / 5 + 365 * counted + counted / 4 - counted / 100 +
      counted / 400 - 32045
  end

  def self.julian_to_jd(year, month, day)
    shift = (14 - month) / 12
    counted = year + 4800 - shift
    moved = month + 12 * shift - 3
    day + (153 * moved + 2) / 5 + 365 * counted + counted / 4 - 32083
  end

  def self.jd_to_gregorian(jd)
    a = jd + 32044
    b = (4 * a + 3) / 146097
    c = a - 146097 * b / 4
    d = (4 * c + 3) / 1461
    e = c - 1461 * d / 4
    m = (5 * e + 2) / 153
    day = e - (153 * m + 2) / 5 + 1
    month = m + 3 - 12 * (m / 10)
    year = 100 * b + d - 4800 + m / 10
    [year, month, day]
  end

  def self.jd_to_julian(jd)
    c = jd + 32082
    d = (4 * c + 3) / 1461
    e = c - 1461 * d / 4
    m = (5 * e + 2) / 153
    day = e - (153 * m + 2) / 5 + 1
    month = m + 3 - 12 * (m / 10)
    year = d - 4800 + m / 10
    [year, month, day]
  end

  # ── What a date stands for ───────────────────────────────────────────────

  def jd
    @jd
  end

  def start
    @start
  end

  # Marshal writes a date as MRI's C library does: the day and the time of
  # day in UTC, the nanoseconds past that second, the offset in seconds,
  # and the calendar reform day as a Float.
  def marshal_dump
    absolute = absolute_day
    utc_jd = absolute.floor
    seconds = (absolute - utc_jd) * 86400
    whole = seconds.floor
    nanoseconds = (seconds - whole) * 1_000_000_000
    nanoseconds = nanoseconds.to_i if nanoseconds.denominator == 1
    reform = if @start.is_a?(Infinity)
      @start.direction * Float::INFINITY
    else
      @start.to_f
    end
    [0, utc_jd, whole, nanoseconds, (offset_fraction * 86400).to_i, reform]
  end

  def marshal_load(dumped)
    raise TypeError, "expected an array" unless dumped.is_a?(Array)
    raise TypeError, "invalid size" unless dumped.size == 6
    _nth, utc_jd, seconds, nanoseconds, offset_seconds, reform = dumped
    offset = Rational(offset_seconds, 86400)
    local = utc_jd + Rational(seconds, 86400) + Rational(nanoseconds, 86400 * 1_000_000_000) + offset
    @jd = local.floor
    if is_a?(DateTime)
      @fraction = local - @jd
      @offset = offset
    end
    @start = if reform.infinite?
      reform.positive? ? JULIAN : GREGORIAN
    else
      reform.to_i
    end
    self
  end

  def julian?
    !Date.gregorian_from?(@jd, @start)
  end

  def gregorian?
    !self.julian?
  end

  def fields
    return @fields unless @fields.nil?
    @fields = self.julian? ? Date.jd_to_julian(@jd) : Date.jd_to_gregorian(@jd)
  end
  private :fields

  # A frozen date has worked out its year, month and day already.
  def freeze
    fields
    super
  end

  def year
    self.fields[0]
  end

  def month
    self.fields[1]
  end

  def mon
    self.fields[1]
  end

  def day
    self.fields[2]
  end

  def mday
    self.fields[2]
  end

  def wday
    (@jd + 1) % 7
  end

  def yday
    first = self.julian? ? Date.julian_to_jd(self.year, 1, 1) : Date.gregorian_to_jd(self.year, 1, 1)
    @jd - first + 1
  end

  def cwday
    counted = self.wday
    counted == 0 ? 7 : counted
  end

  def cweek
    (@jd - self.commercial_monday) / 7 + 1
  end

  def cwyear
    thursday = @jd + (4 - self.cwday)
    held = Date.from_jd(thursday, @start)
    held.year
  end

  # The Monday that opens the commercial year this date falls in.
  def commercial_monday
    thursday = @jd + (4 - self.cwday)
    held = Date.from_jd(thursday, @start)
    fourth = Date.civil(held.year, 1, 4, @start)
    fourth.jd - (fourth.cwday - 1)
  end
  private :commercial_monday

  def mjd
    @jd - 2400001
  end

  def ajd
    absolute_day - Rational(1, 2)
  end

  def amjd
    ajd - Rational(4800001, 2)
  end

  def day_fraction
    @fraction || Rational(0, 1)
  end

  # The clock a date carries. A plain Date stands for midnight with no offset
  # from UTC, and DateTime answers the time of day and offset it holds.
  def clock_fraction
    Rational(0, 1)
  end
  private :clock_fraction

  def zone_offset
    Rational(0, 1)
  end
  private :zone_offset

  def clock_seconds
    self.clock_fraction * 86400
  end
  private :clock_seconds

  def clock_hour
    self.clock_seconds.floor / 3600
  end
  private :clock_hour

  def clock_minute
    self.clock_seconds.floor / 60 % 60
  end
  private :clock_minute

  def clock_second
    self.clock_seconds.floor % 60
  end
  private :clock_second

  def clock_sub_second
    self.clock_seconds - self.clock_seconds.floor
  end
  private :clock_sub_second

  def hour
    clock_hour
  end

  def min
    clock_minute
  end

  def sec
    clock_second
  end
  alias_method :minute, :min
  alias_method :second, :sec
  private :hour, :min, :minute, :sec, :second

  # The part of a second below one, written to as many digits as asked for.
  def sub_second_digits(count)
    (self.clock_sub_second * 10 ** count).floor.to_s.rjust(count, "0")
  end
  private :sub_second_digits

  # The offset from UTC, written with as many separators as the directive
  # asking for it wants.
  def offset_text(separators)
    total = (self.zone_offset * 86400).to_i
    sign = total < 0 ? "-" : "+"
    total = total.abs
    written = "#{sign}#{self.two(total / 3600)}"
    return written + self.two(total / 60 % 60) if separators == 0
    written = written + ":" + self.two(total / 60 % 60)
    return written if separators == 1
    written + ":" + self.two(total % 60)
  end
  private :offset_text

  def epoch_seconds
    ((@jd - 2440588) * 86400 + self.clock_seconds - self.zone_offset * 86400).floor
  end
  private :epoch_seconds

  def epoch_milliseconds
    (((@jd - 2440588) * 86400 + self.clock_seconds -
      self.zone_offset * 86400) * 1000).floor
  end
  private :epoch_milliseconds

  def ld
    @jd - 2299160
  end

  def leap?
    self.julian? ? Date.julian_leap?(self.year) : Date.gregorian_leap?(self.year)
  end

  def sunday?
    self.wday == 0
  end

  def monday?
    self.wday == 1
  end

  def tuesday?
    self.wday == 2
  end

  def wednesday?
    self.wday == 3
  end

  def thursday?
    self.wday == 4
  end

  def friday?
    self.wday == 5
  end

  def saturday?
    self.wday == 6
  end

  # ── Moving about ─────────────────────────────────────────────────────────

  # The same point on the calendar, moved to another day. DateTime overrides
  # this so a shifted one keeps the time of day and offset it holds.
  def rebuild(jd)
    self.class.from_jd(jd, @start)
  end
  private :rebuild

  # This date moved to day `jd`, `fraction` of the way into it. A Date
  # holds a fraction only when arithmetic gave it one.
  def moved_to(jd, fraction)
    made = rebuild(jd)
    made.instance_variable_set(:@fraction, fraction) unless fraction.zero?
    made
  end
  private :moved_to

  def +(count)
    raise TypeError, "expected numeric" unless count.is_a?(Numeric)
    shifted(count)
  end

  def -(other)
    return absolute_day - other.absolute_day if other.is_a?(Date)
    raise TypeError, "expected numeric" unless other.is_a?(Numeric)
    shifted(-other)
  end

  # This date `count` days on, kept whole when the count is.
  def shifted(count)
    return moved_to(@jd + count, day_fraction) if count.is_a?(Integer)
    count = float_days(count) if count.is_a?(Float) && count.finite?

    moved = @jd + day_fraction + count.to_r
    landing = moved.floor
    moved_to(landing, moved - landing)
  end
  private :shifted

  # A Float count of days as whole days, whole seconds and nanoseconds,
  # the nanoseconds rounded from what the Float holds past the second.
  def float_days(count)
    rest = count.abs
    days = rest.floor
    rest = (rest - days) * 86400
    seconds = rest.floor
    nanoseconds = ((rest - seconds) * 1_000_000_000).round
    whole = days + Rational(seconds, 86400) + Rational(nanoseconds, 86_400_000_000_000)
    count < 0 ? -whole : whole
  end
  private :float_days

  # The point in time this date names, counted in days from the same place
  # whatever offset it is written against.
  def absolute_day
    @jd + day_fraction - offset_fraction
  end
  protected :absolute_day

  # The offset from UTC as a fraction of a day, which is none for a Date.
  def offset_fraction
    Rational(0, 1)
  end
  private :offset_fraction

  def >>(months)
    raise TypeError, "expected numeric" unless months.is_a?(Numeric)
    total = self.year * 12 + (self.month - 1) + months
    landing_year = total.div(12)
    landing_month = (total % 12).to_i + 1
    landing_day = self.day
    limit = Date.days_in_month(landing_year, landing_month, @start)
    landing_day = limit if landing_day > limit
    found = Date.valid_civil_jd(landing_year, landing_month, landing_day, @start)
    while found.nil? && landing_day > 1
      landing_day -= 1
      found = Date.valid_civil_jd(landing_year, landing_month, landing_day, @start)
    end
    raise Date::Error, "invalid date" if found.nil?
    self.rebuild(found)
  end

  def <<(months)
    raise TypeError, "expected numeric" unless months.is_a?(Numeric)
    self >> -months
  end

  def next_day(count = 1)
    self + count
  end

  def prev_day(count = 1)
    self - count
  end

  def next_month(count = 1)
    self >> count
  end

  def prev_month(count = 1)
    self << count
  end

  def next_year(count = 1)
    self >> (count * 12)
  end

  def prev_year(count = 1)
    self << (count * 12)
  end

  def succ
    self + 1
  end

  def next
    self + 1
  end

  def upto(limit, &block)
    return to_enum(:upto, limit) if block.nil?
    walked = self
    while walked <= limit
      block.call(walked)
      walked = walked + 1
    end
    self
  end

  def downto(limit, &block)
    return to_enum(:downto, limit) if block.nil?
    walked = self
    while walked >= limit
      block.call(walked)
      walked = walked - 1
    end
    self
  end

  def step(limit, by = 1, &block)
    return to_enum(:step, limit, by) if block.nil?
    raise ArgumentError, "step can't be 0" if by == 0
    walked = self
    if by > 0
      while walked <= limit
        block.call(walked)
        walked = walked + by
      end
    else
      while walked >= limit
        block.call(walked)
        walked = walked + by
      end
    end
    self
  end

  # ── Which calendar a date is read on ─────────────────────────────────────

  def new_start(start = ITALY)
    self.class.from_jd(@jd, start)
  end

  def italy
    new_start(ITALY)
  end

  def england
    new_start(ENGLAND)
  end

  def julian
    new_start(JULIAN)
  end

  def gregorian
    new_start(GREGORIAN)
  end

  # ── Comparing and rendering ──────────────────────────────────────────────

  def <=>(other)
    return absolute_day <=> other.absolute_day if other.is_a?(Date)
    return ajd <=> other if other.is_a?(Numeric)
    nil
  end

  def ===(other)
    return @jd == other.jd if other.is_a?(Date)
    return @jd == other if other.is_a?(Numeric)
    false
  end

  def eql?(other)
    other.is_a?(Date) && absolute_day == other.absolute_day
  end

  def hash
    absolute_day.hash
  end

  def to_s
    "#{self.padded_year}-#{self.two(self.month)}-#{self.two(self.day)}"
  end

  def inspect
    "#<#{self.class}: #{to_s} #{inspected_parts(@jd, day_fraction, 0)}>"
  end

  # The moment as Ruby's own inspect spells it: the Julian day and the
  # seconds and nanoseconds into it in UTC, the offset in seconds, and the
  # day the Gregorian calendar starts.
  def inspected_parts(jd, fraction, offset)
    utc = fraction - offset
    day = utc.floor
    seconds = (utc - day) * 86400
    whole = seconds.floor
    nanoseconds = ((seconds - whole) * 1_000_000_000).floor
    offset_seconds = (offset * 86400).round
    sign = offset_seconds < 0 ? "-" : "+"
    start = if @start.is_a?(Date::Infinity) || (@start.is_a?(Float) && @start.infinite?)
              (@start < 0 ? "-Inf" : "Inf")
            else
              @start.to_i.to_s
            end
    "((#{jd + day}j,#{whole}s,#{nanoseconds}n),#{sign}#{offset_seconds.abs}s,#{start}j)"
  end
  private :inspected_parts

  def iso8601
    to_s
  end

  def xmlschema
    to_s
  end

  def rfc3339
    "#{to_s}T00:00:00+00:00"
  end

  # The Japanese eras by the day each began, with the letter it is written
  # with and the year before its first.
  JAPANESE_ERAS = [
    [2_458_605, "R", 2018], [2_447_535, "H", 1988], [2_424_875, "S", 1925],
    [2_419_614, "T", 1911], [2_405_160, "M", 1867]
  ].freeze
  private_constant :JAPANESE_ERAS

  def jisx0301
    era = JAPANESE_ERAS.find { |began, _, _| jd >= began }
    return iso8601 unless era

    format("%s%02d.%02d.%02d", era[1], year - era[2], month, day)
  end

  def asctime
    "#{ABBR_DAYNAMES[self.wday]} #{ABBR_MONTHNAMES[self.month]} #{self.day.to_s.rjust(2)} #{clock_text} #{self.padded_year}"
  end

  # A Date is at midnight.
  def clock_text
    "00:00:00"
  end
  private :clock_text

  def ctime
    asctime
  end

  def to_date
    self
  end

  def to_datetime
    DateTime.from_parts(@jd, Rational(0, 1), Rational(0, 1), @start)
  end

  def to_time
    Time.local(self.year, self.month, self.day)
  end

  def deconstruct_keys(keys)
    all = { year: self.year, month: self.month, day: self.day, yday: self.yday, wday: self.wday }
    return all if keys.nil?
    unless keys.is_a?(Array)
      raise TypeError, "wrong argument type #{keys.class} (expected Array or nil)"
    end
    picked = {}
    keys.each { |key| picked[key] = all[key] if all.key?(key) }
    picked
  end

  def strftime(template = "%F")
    template = template.to_s
    built = ""
    index = 0
    while index < template.length
      character = template[index]
      unless character == "%"
        built = built + character
        index = index + 1
        next
      end
      index = index + 1
      flags = ""
      while index < template.length && "-_0^#".include?(template[index])
        flags = flags + template[index]
        index = index + 1
      end
      width = ""
      while index < template.length && template[index] =~ /\A[0-9]\z/
        width = width + template[index]
        index = index + 1
      end
      if index >= template.length
        built = built + "%" + flags + width
        break
      end
      directive = template[index]
      if directive == ":"
        ahead = index
        ahead += 1 while ahead < template.length && template[ahead] == ":"
        if ahead < template.length && template[ahead] == "z"
          directive = ahead - index == 1 ? ":z" : "::z"
          index = ahead
        end
      end
      index = index + 1
      built = built + self.format_directive(directive, flags, width)
    end
    built
  end

  # One `%` directive of a strftime template, with the GNU flags that pad it,
  # drop its zeros, or shift its case applied.
  def format_directive(directive, flags, width)
    return self.sub_second_digits(width.empty? ? 9 : width.to_i) if directive == "N"
    body = self.directive_body(directive)
    return body if body.nil? == false && self.directive_is_literal(directive)
    return "%" + flags + width + directive if body.nil?
    body = body.upcase if flags.include?("^")
    if flags.include?("-")
      # The `-` flag drops the padding a directive is written with, leaving
      # one character behind when the whole of it is padding.
      leading = body[/\A[0 ]+/]
      return body if leading.nil?
      kept = leading.length == body.length ? body.length - 1 : leading.length
      return body[kept..-1]
    end
    filler = self.padding_character(directive, flags)
    wanted = width.empty? ? self.directive_width(directive) : width.to_i
    return body if body.length >= wanted
    if filler == "0" && (body.start_with?("-") || body.start_with?("+"))
      return body[0] + body[1..-1].rjust(wanted - 1, "0")
    end
    body.rjust(wanted, filler)
  end
  private :format_directive

  # Whether a directive writes text of its own that no flag or width touches.
  def directive_is_literal(directive)
    ["n", "t", "%", "L"].include?(directive)
  end
  private :directive_is_literal

  # The last of the `0` and `_` flags decides what pads a directive, so
  # `%0_10h` pads with spaces and `%_010h` pads with zeros.
  def padding_character(directive, flags)
    picked = nil
    flags.each_char do |flag|
      picked = "0" if flag == "0"
      picked = " " if flag == "_"
    end
    return picked unless picked.nil?
    ["e", "k", "l"].include?(directive) ? " " : self.default_filler(directive)
  end
  private :padding_character

  def default_filler(directive)
    ["A", "a", "B", "b", "h", "P", "p", "Z", "c", "v", "+", "z", ":z",
     "::z"].include?(directive) ? " " : "0"
  end
  private :default_filler

  def directive_width(directive)
    return 3 if directive == "j"
    return 4 if ["Y", "G"].include?(directive)
    return 1 if ["u", "w", "n", "t", "%", "L", "N", "s", "Q"].include?(directive)
    return 0 if ["A", "a", "B", "b", "h", "P", "p", "Z", "c", "D", "F", "R", "r",
                 "T", "X", "x", "v", "+", "z", ":z", "::z"].include?(directive)
    2
  end
  private :directive_width

  def directive_body(directive)
    case directive
    when "A" then DAYNAMES[self.wday]
    when "a" then ABBR_DAYNAMES[self.wday]
    when "B" then MONTHNAMES[self.month]
    when "b", "h" then ABBR_MONTHNAMES[self.month]
    when "C" then (self.year / 100).to_s
    when "d" then self.day.to_s
    when "e" then self.day.to_s
    when "F" then self.strftime("%Y-%m-%d")
    when "G" then self.cwyear.to_s
    when "g" then (self.cwyear % 100).to_s
    when "H", "k" then self.clock_hour.to_s
    when "I", "l" then self.twelve_hour.to_s
    when "j" then self.yday.to_s
    when "M" then self.clock_minute.to_s
    when "m" then self.month.to_s
    when "n" then "\n"
    when "P" then self.clock_hour < 12 ? "am" : "pm"
    when "p" then self.clock_hour < 12 ? "AM" : "PM"
    when "S" then self.clock_second.to_s
    when "s" then self.epoch_seconds.to_s
    when "L" then self.sub_second_digits(3)
    when "Q" then self.epoch_milliseconds.to_s
    when "t" then "\t"
    when "U" then self.week_of_year(0).to_s
    when "W" then self.week_of_year(1).to_s
    when "u" then self.cwday.to_s
    when "V" then self.cweek.to_s
    when "w" then self.wday.to_s
    when "Y" then self.year.to_s
    when "y" then (self.year % 100).to_s
    when "Z", ":z" then self.offset_text(1)
    when "::z" then self.offset_text(2)
    when "z" then self.offset_text(0)
    when "%" then "%"
    when "c" then self.strftime("%a %b %e %H:%M:%S %Y")
    when "D", "x" then self.strftime("%m/%d/%y")
    when "R" then self.strftime("%H:%M")
    when "r" then self.strftime("%I:%M:%S %p")
    when "T", "X" then self.strftime("%H:%M:%S")
    when "v" then self.strftime("%e-%^b-%Y")
    when "+" then self.strftime("%a %b %e %H:%M:%S %Z %Y")
    end
  end
  private :directive_body

  # The hour a twelve-hour clock shows, where midnight and noon both read 12.
  def twelve_hour
    shown = self.clock_hour % 12
    shown == 0 ? 12 : shown
  end
  private :twelve_hour

  # The week of the year this date falls in, counting from the first week that
  # opens on the named weekday.
  def week_of_year(opening)
    first = @jd - self.yday + 1
    offset = (first + 1) % 7
    shift = (opening - offset) % 7
    week_one = first + shift
    return 0 if @jd < week_one
    (@jd - week_one) / 7 + 1
  end
  private :week_of_year

  # The year written to four digits at least, with a sign when it falls
  # before the common era.
  def padded_year
    counted = self.year
    return "-#{counted.abs.to_s.rjust(4, "0")}" if counted < 0
    counted.to_s.rjust(4, "0")
  end
  private :padded_year

  def two(field)
    field.to_s.rjust(2, "0")
  end
  private :two
end

# A calendar day together with a time of day and an offset from UTC. The date
# is held the way Date holds one, and the time as an exact fraction of a day.
class DateTime < Date
  def self.from_parts(jd, fraction, offset, start)
    made = allocate
    made.instance_variable_set(:@jd, jd)
    made.instance_variable_set(:@fraction, fraction)
    made.instance_variable_set(:@offset, offset)
    made.instance_variable_set(:@start, start)
    made
  end

  def self.from_jd(jd, start)
    from_parts(jd, Rational(0, 1), Rational(0, 1), start)
  end

  # ── Building ─────────────────────────────────────────────────────────────

  def self.civil(*args)
    (year, month, day, hour, minute, second), offset, start, fraction =
      read_clock_arguments(args, [[-4712, "year", :any], [1, "month", nil], [1, "day", 1]])
    found = valid_civil_jd(year, month, day, start)
    raise Date::Error, "invalid date" if found.nil?
    with_clock(found, hour, minute, second, offset, start, fraction)
  end

  def self.new(*args)
    civil(*args)
  end

  # A Julian Day Number with a fraction names the time of day too.
  def self.jd(*args)
    (number, hour, minute, second), offset, start, fraction =
      read_clock_arguments(args, [[0, "jd", 1]])
    with_clock(number, hour, minute, second, offset, start, fraction)
  end

  def self.ordinal(*args)
    (year, day, hour, minute, second), offset, start, fraction =
      read_clock_arguments(args, [[-4712, "year", :any], [1, "yday", 1]])
    found = valid_ordinal_jd(year, day, start)
    raise Date::Error, "invalid date" if found.nil?
    with_clock(found, hour, minute, second, offset, start, fraction)
  end

  def self.commercial(*args)
    (year, week, weekday, hour, minute, second), offset, start, fraction =
      read_clock_arguments(args, [[-4712, "year", :any], [1, "cweek", nil], [1, "cwday", 1]])
    found = Date.commercial(year, week, weekday, start)
    with_clock(found.jd, hour, minute, second, offset, start, fraction)
  end

  # A constructor's arguments read as MRI reads them: the date fields named
  # in `leading`, each with its default, its name, and the parts of a day
  # one of its units is, then hour, minute, second, offset and start. A
  # field with a fraction must be the last one given before the second,
  # and the fraction is carried as a part of a day.
  def self.read_clock_arguments(args, leading)
    count = args.length
    fields = leading + [[0, "hour", 24], [0, "minute", 1440], [0, "second", 86400]]
    limit = fields.length + 2
    if count > limit
      raise ArgumentError, "wrong number of arguments (given #{count}, expected 0..#{limit})"
    end
    values = fields.each_with_index.map { |(default, _, _), index| count > index ? args[index] : default }
    fraction = 0
    (fields.length - 1).downto(0) do |index|
      next unless count > index

      _, name, per_day = fields[index]
      value = values[index]
      raise TypeError, "invalid #{name} (not numeric)" unless value.is_a?(Numeric)
      next if per_day == :any

      if per_day.nil?
        values[index] = value.to_i
        next
      end
      values[index], part = clock_trunc(value, per_day)
      next if part == 0
      if name != "second" && count > index + 1
        raise Date::Error, "invalid fraction"
      end
      fraction = part
    end
    offset = count > fields.length ? args[fields.length] : 0
    start = count > fields.length + 1 ? args[fields.length + 1] : ITALY
    [values, offset, start, fraction]
  end
  private_class_method :read_clock_arguments

  # The whole units of a reading, floored, and what is left past them as a
  # part of a day.
  def self.clock_trunc(value, per_day)
    whole = value.is_a?(Integer) ||
            (value.is_a?(Float) && value.round == value) ||
            (value.is_a?(Rational) && value.denominator == 1)
    return [value.to_i, 0] if whole

    [value.div(1), (value % 1) / per_day]
  end
  private_class_method :clock_trunc

  def self.now(start = ITALY)
    moment = Time.now
    found = Date.civil(moment.year, moment.month, moment.day, start)
    fraction = Rational(moment.hour * 3600 + moment.min * 60 + moment.sec, 86400) +
      moment.subsec / 86400
    from_parts(found.jd, fraction, Rational(moment.utc_offset, 86400), start)
  end

  # A time of day fixed to a Julian Day Number, with `fraction` of a day
  # added. An hour, minute, or second written as a negative counts back
  # from the next unit up, and 24 hours names midnight on the following day.
  def self.with_clock(jd, hour, minute, second, offset, start, fraction = 0)
    hour += 24 if hour < 0
    minute += 60 if minute < 0
    second += 60 if second < 0
    if hour < 0 || hour > 24 || minute < 0 || minute > 59 || second < 0 || second > 59 ||
       (hour == 24 && (minute > 0 || second > 0))
      raise Date::Error, "invalid date"
    end
    if hour == 24
      hour = 0
      fraction += 1
    end
    made = from_parts(jd, Rational(hour * 3600 + minute * 60 + second, 86400),
                      read_offset(offset), start)
    fraction == 0 ? made : made + fraction
  end
  private_class_method :with_clock

  # An offset from UTC, given either as a fraction of a day or as the text a
  # zone is written with.
  def self.read_offset(offset)
    return offset.to_r if offset.is_a?(Numeric)
    raise Date::Error, "invalid date" unless offset.is_a?(String)
    seconds = Date.send(:zone_to_diff, offset)
    unless seconds.is_a?(Integer) && seconds.abs <= 86400
      warn "invalid offset is ignored" if $VERBOSE
      return Rational(0, 1)
    end
    Rational(seconds, 86400)
  end

  # ── Reading text ─────────────────────────────────────────────────────────

  def self.parse(text = "-4712-01-01T00:00:00+00:00", comp = true, start = ITALY)
    new_by_frags(_parse(text, comp), start)
  end

  def self._strptime(text, format = "%FT%T%z")
    read_strptime(text, format)
  end

  def self.strptime(text = "-4712-01-01T00:00:00+00:00", format = "%FT%T%z",
                    start = ITALY)
    new_by_frags(_strptime(text, format), start)
  end

  # The datetime a set of fields read from text names, filling what the
  # text left out from today, at the time of day and offset it gives.
  def self.new_by_frags(fields, start)
    raise Date::Error, "invalid date" if fields.nil?
    if civil_fields?(fields)
      jd = valid_civil_jd(fields[:year], fields[:mon], fields[:mday], start)
      fields = fields.dup
      fields[:hour] = 0 if fields[:hour].nil?
      fields[:min] = 0 if fields[:min].nil?
      if fields[:sec].nil?
        fields[:sec] = 0
      elsif fields[:sec] == 60
        fields[:sec] = 59
      end
    else
      fields = complete_frags(rewrite_frags(fields))
      jd = valid_date_frags(fields, start)
    end
    raise Date::Error, "invalid date" if jd.nil?

    hour = fields[:hour]
    minute = fields[:min]
    second = fields[:sec]
    hour += 24 if hour < 0
    minute += 60 if minute < 0
    second += 60 if second < 0
    if hour < 0 || hour > 24 || minute < 0 || minute > 59 || second < 0 || second > 59 ||
       (hour == 24 && (minute > 0 || second > 0))
      raise Date::Error, "invalid date"
    end
    if hour == 24
      jd += 1
      hour = 0
    end
    offset = fields[:offset].nil? ? 0 : fields[:offset].to_i
    if offset < -86400 || offset > 86400
      warn "invalid offset is ignored" if $VERBOSE
      offset = 0
    end
    fraction = Rational(hour * 3600 + minute * 60 + second, 86400) +
               (fields[:sec_fraction] || 0).to_r / 86400
    from_parts(jd, fraction, Rational(offset, 86400), start)
  end
  private_class_method :new_by_frags

  def self.iso8601(text = "-4712-01-01T00:00:00+00:00", start = ITALY, limit: 128)
    read_written(text, start) { |held| Date._iso8601(held) }
  end

  def self.rfc3339(text = "-4712-01-01T00:00:00+00:00", start = ITALY, limit: 128)
    read_written(text, start) { |held| Date._rfc3339(held) }
  end

  def self.xmlschema(text = "-4712-01-01T00:00:00+00:00", start = ITALY, limit: 128)
    read_written(text, start) { |held| Date._xmlschema(held) }
  end

  def self.jisx0301(text = "-4712-01-01T00:00:00+00:00", start = ITALY)
    iso8601(text, start)
  end

  def self.httpdate(text = "Mon, 01 Jan -4712 00:00:00 GMT", start = ITALY)
    read_written(text, start) { |held| Date._parse(held) }
  end

  def self.rfc2822(text = "Mon, 1 Jan -4712 00:00:00 +0000", start = ITALY)
    read_written(text, start) { |held| Date._parse(held) }
  end

  def self.rfc822(text = "Mon, 1 Jan -4712 00:00:00 +0000", start = ITALY)
    rfc2822(text, start)
  end

  # Text read into a date by the block, with the time of day and offset the
  # same text carries laid over it.
  def self.read_written(text, start)
    raise Date::Error, "invalid date" if text.nil?
    fields = yield(text)
    new_by_frags(fields, start)
  end

  # ── What a datetime stands for ───────────────────────────────────────────

  def clock_fraction
    @fraction
  end
  private :clock_fraction

  def zone_offset
    @offset
  end
  private :zone_offset

  def clock_text
    format("%02d:%02d:%02d", self.hour, self.min, self.sec)
  end
  private :clock_text

  def hour
    self.clock_hour
  end

  def min
    self.clock_minute
  end

  def minute
    self.clock_minute
  end

  def sec
    self.clock_second
  end

  def second
    self.clock_second
  end

  def sec_fraction
    self.clock_sub_second
  end

  def second_fraction
    self.clock_sub_second
  end

  def offset
    @offset
  end

  def day_fraction
    @fraction
  end

  def zone
    self.offset_text(1)
  end

  # The same instant, read against another offset from UTC.
  def new_offset(offset = 0)
    wanted = DateTime.read_offset(offset)
    moved = @jd + @fraction - @offset + wanted
    landing = moved.floor
    self.class.from_parts(landing, moved - landing, wanted, @start)
  end

  def new_start(start = ITALY)
    self.class.from_parts(@jd, @fraction, @offset, start)
  end

  def rebuild(jd)
    self.class.from_parts(jd, @fraction, @offset, @start)
  end
  private :rebuild

  # ── Moving about ─────────────────────────────────────────────────────────

  def moved_to(jd, fraction)
    self.class.from_parts(jd, fraction, @offset, @start)
  end
  private :moved_to

  def offset_fraction
    @offset
  end
  private :offset_fraction

  # ── Writing text ─────────────────────────────────────────────────────────

  def strftime(template = "%FT%T%:z")
    super(template)
  end

  def to_s
    self.strftime("%Y-%m-%dT%H:%M:%S%:z")
  end

  def inspect
    "#<#{self.class}: #{to_s} #{inspected_parts(@jd, @fraction, @offset)}>"
  end

  def iso8601(digits = 0)
    strftime("%FT%T#{written_fraction(digits)}%:z")
  end

  def xmlschema(digits = 0)
    iso8601(digits)
  end

  def rfc3339(digits = 0)
    iso8601(digits)
  end

  def jisx0301(digits = 0)
    "#{super()}#{strftime("T%T#{written_fraction(digits)}%:z")}"
  end

  # The fraction of a second as `digits` digits after a point, or nothing
  # for none.
  def written_fraction(digits)
    digits = Integer(digits)
    digits.positive? ? ".%#{digits}N" : ""
  end
  private :written_fraction

  def httpdate
    self.new_offset(0).strftime("%a, %d %b %Y %H:%M:%S GMT")
  end

  def rfc2822
    self.strftime("%a, %-d %b %Y %H:%M:%S %z")
  end

  def rfc822
    self.rfc2822
  end

  def to_date
    Date.from_jd(@jd, @start)
  end

  def to_datetime
    self
  end

  def to_time
    parts = Date.jd_to_gregorian(@jd)
    Time.new(parts[0], parts[1], parts[2], self.hour, self.min,
             self.sec + self.sec_fraction, (@offset * 86400).to_i)
  end

  def deconstruct_keys(keys)
    all = { year: self.year, month: self.month, day: self.day, yday: self.yday,
            wday: self.wday, hour: self.hour, min: self.min, sec: self.sec,
            sec_fraction: self.sec_fraction, zone: self.zone }
    return all if keys.nil?
    unless keys.is_a?(Array)
      raise TypeError, "wrong argument type #{keys.class} (expected Array or nil)"
    end
    picked = {}
    keys.each { |key| picked[key] = all[key] if all.key?(key) }
    picked
  end
end

class Time
  # The same instant as a DateTime, read on the calendar the offset this Time
  # carries puts it on.
  def to_datetime
    seconds = self.to_r + self.utc_offset
    days = seconds / 86400
    landing = days.floor
    DateTime.from_parts(landing + 2440588, days - landing,
                        Rational(self.utc_offset, 86400), Date::ITALY)
  end

  def to_date
    days = (self.to_r + self.utc_offset) / 86400
    Date.from_jd(days.floor + 2440588, Date::ITALY)
  end
end
