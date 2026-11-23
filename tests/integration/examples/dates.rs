// Examples covering Time

use super::run_example;

/// The expected output of both `dates/points_in_time` variants, which differ
/// only in whether the calls are written with parentheses.
const POINTS_IN_TIME_OUTPUT: &str = "2007\n11\n1\n15\n25\n30\n4\n305\n123456\n123456000\n(1929/15625)\ntrue\n\"UTC\"\n0\n1193930730\n1193930730.123456\n(18655167658179/15625)\n[30, 25, 15, 1, 11, 2007, 4, 305, false, \"UTC\"]\n\"2007-11-01 15:25:30.123456 UTC\"\n\"2007-11-01 15:25:30 UTC\"\n\"Thu Nov  1 15:25:30 2007\"\n\"2007-11-01T15:25:30Z\"\n\"2007-11-01T15:25:30.123Z\"\n\"2007/11/01 15:25:30\"\n{year: 2007, month: 11, day: 1}\n100.0001\n(1/2)\n(3/4)\n1.0\n101\ntrue\ntrue\ntrue\nfalse\ntrue\n482196050\n482196050\n7245\nnil\n1\n";

#[test]
fn test_dates_points_in_time_execution() {
    let output = run_example("dates/points_in_time.rb");
    assert_eq!(output, POINTS_IN_TIME_OUTPUT);
}

#[test]
fn test_dates_points_in_time_no_parens_execution() {
    let output = run_example("dates/points_in_time_no_parens.rb");
    assert_eq!(output, POINTS_IN_TIME_OUTPUT);
}

/// The expected output of both `dates/calendar_days` variants, which differ
/// only in whether the calls are written with parentheses.
const CALENDAR_DAYS_OUTPUT: &str = "2454482\n2008\n1\n16\n16\n3\n2008\n3\n3\n54481\n(4908963/2)\ntrue\ntrue\n\"2008-01-16\"\n\"#<Date: 2008-01-16 ((2454482j,0s,0n),+0s,2299161j)>\"\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\n\"2008-02-15\"\n\"2007-12-17\"\n\"2008-03-16\"\n\"2007-12-16\"\n\"2008-01-17\"\n\"2008-02-16\"\n\"2009-01-16\"\n(29/1)\ntrue\ntrue\nfalse\ntrue\ntrue\n\"Wednesday, January 16, 2008\"\n\"Wed Jan 16 2008\"\n\"016 02 02 03\"\n\"       JAN\"\n\"1582-10-04\"\n\"1582-10-15\"\ntrue\ntrue\ntrue\n[16, 17, 18, 19]\n[16, 18, 20, 22]\n";

#[test]
fn test_dates_calendar_days_execution() {
    let output = run_example("dates/calendar_days.rb");
    assert_eq!(output, CALENDAR_DAYS_OUTPUT);
}

#[test]
fn test_dates_calendar_days_no_parens_execution() {
    let output = run_example("dates/calendar_days_no_parens.rb");
    assert_eq!(output, CALENDAR_DAYS_OUTPUT);
}

/// The expected output of both `dates/clock_and_calendar` variants, which
/// differ only in whether the calls are written with parentheses.
const CLOCK_AND_CALENDAR_OUTPUT: &str = "2012\n12\n24\n13\n45\n30\n(0/1)\n(1/8)\n\"+03:00\"\n\"2012-12-24T13:45:30+03:00\"\n(1651/2880)\n\"2012-12-24T05:45:30-05:00\"\n\"2012-12-24T10:45:30+00:00\"\n\"2012-12-24\"\nDate\n\"2012-12-24T00:00:00+00:00\"\n\"2012-12-24 13:45:30 +0300\"\n\"01:45 PM\"\n\"Monday, 24 December 2012\"\n\"+03:00:00\"\n\"04:05:06.250000\"\n\"250\"\n\"2012-11-08T15:43:59+00:00\"\n\"2012-11-08T15:43:59+00:00\"\n\"2018-01-01T00:00:00+00:00\"\n\"2012-12-24T19:45:30+03:00\"\n\"2012-12-24T01:45:30+03:00\"\n(1/1)\n(1/4)\ntrue\n25\n0\n\"2012-12-31T23:58:59+00:00\"\n\"2012-12-31\"\n\"2012-12-31 23:58:59 UTC\"\n";

#[test]
fn test_dates_clock_and_calendar_execution() {
    let output = run_example("dates/clock_and_calendar.rb");
    assert_eq!(output, CLOCK_AND_CALENDAR_OUTPUT);
}

#[test]
fn test_dates_clock_and_calendar_no_parens_execution() {
    let output = run_example("dates/clock_and_calendar_no_parens.rb");
    assert_eq!(output, CLOCK_AND_CALENDAR_OUTPUT);
}

/// The expected output of both `dates/zone_names/utc` variants.
const ZONE_NAMES_OUTPUT: &str = concat!(
    "\"UTC\"\n",
    "\"UTC\"\n",
    "\"UTC\"\n",
    "\"UTC\"\n",
    "\"UTC\"\n",
    "\"UTC\"\n",
    "#<Encoding:US-ASCII>\n",
    "true\n",
    "nil\n"
);

#[test]
fn test_dates_zone_names_utc_execution() {
    let output = run_example("dates/zone_names/utc.rb");
    assert_eq!(output, ZONE_NAMES_OUTPUT);
}

#[test]
fn test_dates_zone_names_utc_parens_execution() {
    let output = run_example("dates/zone_names/utc_parens.rb");
    assert_eq!(output, ZONE_NAMES_OUTPUT);
}

/// The expected output of both `dates/times_from_parts` variants, which differ only in whether the
/// calls are written with parentheses.
const TIMES_FROM_PARTS_OUTPUT: &str = concat!(
    "2000-01-01 20:15:01 UTC\n",
    "2000-08-08 08:08:08 UTC\n",
    "2000-12-01 00:00:00 UTC\n",
    "2000-01-01 20:15:01 UTC\n",
    "1\n",
    "2\n",
    "argument out of range\n",
);

#[test]
fn test_dates_times_from_parts_execution() {
    let output = run_example("dates/times_from_parts.rb");
    assert_eq!(output, TIMES_FROM_PARTS_OUTPUT);
}

#[test]
fn test_dates_times_from_parts_no_parens_execution() {
    let output = run_example("dates/times_from_parts_no_parens.rb");
    assert_eq!(output, TIMES_FROM_PARTS_OUTPUT);
}

/// The expected output of both `dates/marshal_bytes` variants, which differ
/// only in whether the calls are written with parentheses.
const MARSHAL_BYTES_OUTPUT: &str = "[75, 0, 25, 192, 0, 0, 64, 133]\n1\n1\n2000\n33\n20\n0\n";

#[test]
fn test_dates_marshal_bytes_execution() {
    let output = run_example("dates/marshal_bytes.rb");
    assert_eq!(output, MARSHAL_BYTES_OUTPUT);
}

#[test]
fn test_dates_marshal_bytes_parens_execution() {
    let output = run_example("dates/marshal_bytes_parens.rb");
    assert_eq!(output, MARSHAL_BYTES_OUTPUT);
}

/// The expected output of both `dates/zone_readings` variants, which differ only in
/// whether the calls are written with parentheses.
const ZONE_READINGS_OUTPUT: &str = "\"2001-02-03 04:05:06\"\n\" 3-FEB-2001\"\n\"Sat Feb  3 04:05:06 2001\"\n\"2/3/1\"\n\"FEB and 00FEB and   Feb\"\n\"034 04 05 05 2001\"\n\"+0000 +00:00 +00:00:00\"\n\"04am  4AM\"\n19800\n\"+0530\"\n-36000\n3600\n\"2020-12-24 15:56:17\"\n(1929/15625)\n\"2021-01-01\"\n3600\n\"FIX\"\n";

#[test]
fn test_dates_zone_readings_execution() {
    let output = run_example("dates/zone_readings.rb");
    assert_eq!(output, ZONE_READINGS_OUTPUT);
}

#[test]
fn test_dates_zone_readings_parens_execution() {
    let output = run_example("dates/zone_readings_parens.rb");
    assert_eq!(output, ZONE_READINGS_OUTPUT);
}

/// The expected output of both `dates/time_zone_parsing` variants.
const TIME_ZONE_PARSING_OUTPUT: &str = "true\nfalse\ntrue\ntrue\ntrue\n\"Thu, 01 Jan 1970 00:00:00 -0000\"\n\"Thu, 01 Jan 1970 05:30:00 +0530\"\n";

#[test]
fn test_dates_time_zone_parsing_execution() {
    let output = run_example("dates/time_zone_parsing.rb");
    assert_eq!(output, TIME_ZONE_PARSING_OUTPUT);
}

#[test]
fn test_dates_time_zone_parsing_no_parens_execution() {
    let output = run_example("dates/time_zone_parsing_no_parens.rb");
    assert_eq!(output, TIME_ZONE_PARSING_OUTPUT);
}

#[test]
fn a_frozen_time_answers_its_readers() {
    let output = run_example("dates/frozen_time_readers.rb");
    assert_eq!(output, "true\n[24, 10, 2026]\n2026-10-24 00:00:00 UTC\n");
}

#[test]
fn time_strptime_reads_a_time_in_the_format_given() {
    let output = run_example("dates/time_strptime.rb");
    assert_eq!(
        output,
        "1996-07-17 02:44:25 -0700\n-25200\nfalse\n1792800000\ntrue\n\"invalid date or strptime format - 'bad' '%Y-%m-%d'\"\n{seconds: 1792800000}\n{year: 2026, mon: 10, mday: 21, zone: \"UTC\", offset: 0}\n\"2026-10-24\"\n\"2026-10-24T00:00:00+00:00\"\n"
    );
}

#[test]
fn strptime_matches_literal_quotes_after_a_zone_and_inspect_shows_utc_parts() {
    let output = run_example("dates/strptime_with_quotes_and_padding.rb");
    assert_eq!(
        output,
        "1996-07-17 02:44:25 -0700\n2020-11-07 01:02:03 -0400\n#<DateTime: 2020-11-07T01:02:03-04:00 ((2459161j,18123s,0n),-14400s,2299161j)>\n#<DateTime: 2020-11-07T01:02:03-04:00 ((2459161j,18123s,0n),-14400s,2299161j)>\n{mday: 7, mon: 11, year: 2020}\n{mday: 7}\n#<Date: 2020-11-07 ((2459161j,0s,0n),+0s,2299161j)>\n#<Date: 2020-11-07 ((2459161j,0s,0n),+0s,-Infj)>\n#<DateTime: 2020-11-07T01:02:03-04:00 ((2459161j,18123s,500000000n),-14400s,2299161j)>\n#<DateTime: 2020-11-07T23:00:00+09:00 ((2459161j,50400s,0n),+32400s,2299161j)>\n#<DateTime: 1582-10-15T00:00:00+00:00 ((2299171j,0s,0n),+0s,2361222j)>\n"
    );
}

#[test]
fn strftime_pads_a_negative_year_and_honors_width_and_spaces() {
    let output = run_example("dates/negative_year_strftime.rb");
    assert_eq!(
        output,
        "\"-0001|-0001|-001|-1|   -1|-00001\"\n\"-12345\"\n\"12345|12345\"\n\"0005|5|   5\"\n"
    );
}

#[test]
fn time_parse_keeps_the_zone_and_fraction_the_text_names() {
    let output = run_example("dates/time_parse_utc_designator.rb");
    assert_eq!(
        output,
        "{hour: 15, mday: 8, min: 43, mon: 11, offset: 0, sec: 59, year: 2012, zone: \"Z\"}\n0\n#<NoMethodError: private method 'zone_utc?' called for class Time>\ntrue\n2012-11-08 15:43:59 +0000\n\"2012-11-08 15:43:59 UTC\"\n\"2012-11-08 15:43:59 +0900\"\n[2, (1/4)]\n250000\n(1/4)\n"
    );
}

#[test]
fn a_day_month_year_date_with_dashes_keeps_its_year_positive() {
    let output = run_example("dates/time_parse_dashed_rfc_date.rb");
    assert_eq!(output, "2037\nfalse\n[2037, 9, 27]\n[2037, 9, 27]\n");
}

#[test]
fn a_date_or_datetime_subclass_keeps_its_class() {
    let output = run_example("dates/datetime_subclass_keeps_its_class.rb");
    assert_eq!(
        output,
        concat!(
            "StampedDateTime\n",
            "\"2000-01-01T06:00:00+00:00\"\n",
            "StampedDateTime\n",
            "StampedDateTime\n",
            "StampedDateTime\n",
            "StampedDateTime\n",
            "StampedDateTime\n",
            "StampedDateTime\n",
            "\"2000-01-01T07:00:00+01:00\"\n",
            "StampedDateTime\n",
            "StampedDateTime\n",
            "StampedDateTime\n",
            "StampedDateTime\n",
            "StampedDateTime\n",
            "StampedDateTime\n",
            "StampedDateTime\n",
            "5\n",
            "StampedDateTime\n",
            "WorkDay\n",
            "WorkDay\n",
            "\"2024-02-29\"\n",
            "WorkDay\n",
            "WorkDay\n",
            "WorkDay\n",
            "Date\n",
            "StampedDateTime\n",
        )
    );
}

#[test]
fn date_and_datetime_differences_count_the_instants_apart() {
    let output = run_example("dates/datetime_difference_across_offsets.rb");
    assert_eq!(
        output,
        concat!(
            "(1/86400)\n",
            "(-1/86400)\n",
            "(79201/86400)\n",
            "(1/2)\n",
            "(1/4)\n",
            "(-11/24)\n",
            "\"2017-01-14T23:00:01+01:00\"\n",
            "(15/1)\n",
            "false\n",
            "10\n",
            "false\n",
        )
    );
}

#[test]
fn iso_week_dates_parse_into_commercial_fields() {
    let output = run_example("dates/iso_week_dates_parsed.rb");
    assert_eq!(
        output,
        concat!(
            "[\"2020-W01-2\", {cwyear: 2020, cweek: 1, cwday: 2}, \"2019-12-31\"]\n",
            "[\"2020-W01\", {cwyear: 2020, cweek: 1}, \"2019-12-30\"]\n",
            "[\"20W012\", {cwyear: 2020, cweek: 1, cwday: 2}, \"2019-12-31\"]\n",
            "[\"2020W012\", {cwyear: 2020, cweek: 1, cwday: 2}, \"2019-12-31\"]\n",
            "[\"2020-w10-7 10:00\", {hour: 10, min: 0, cwyear: 2020, cweek: 10, cwday: 7}, \"2020-03-08\"]\n",
            "[\"Week 2020-W01-2 ok\", {cwyear: 2020, cweek: 1, cwday: 2}, \"2019-12-31\"]\n",
            "[\"20-W01\", {cwyear: 2020, cweek: 1}, \"2019-12-30\"]\n",
            "{cwday: 3}\n",
            "{cweek: 5}\n",
            "3\n",
            "5\n",
            "\"2019-12-30T10:30:00+00:00\"\n",
            "[ArgumentError, \"no time information in \\\"2020-W01-2\\\"\"]\n",
            "{year: 2024, mon: 3, mday: 9}\n",
        )
    );
}

#[test]
fn datetime_ctime_writes_its_time_of_day() {
    let output = run_example("dates/datetime_ctime_keeps_the_clock.rb");
    assert_eq!(
        output,
        "\"Fri Nov  6 19:06:45 2026\"\n\"Fri Nov  6 19:06:45 2026\"\n\"Fri Nov  6 00:00:00 2026\"\n\"Fri Nov  6 00:00:00 2026\"\n"
    );
}

#[test]
fn iso8601_cuts_off_extra_digits() {
    let output = run_example("dates/iso8601_cuts_off_extra_digits.rb");
    assert_eq!(
        output,
        "\"2024-01-02T03:04:05.678Z\"\n\"678\"\n(6789/10000)\n678900000\n\"1970-01-01T00:00:00.999Z\"\n"
    );
}

#[test]
fn test_parsing_without_a_year_does_not_call_a_replaced_today() {
    let output = run_example("dates/parse_without_a_year_after_today_is_replaced.rb");
    assert_eq!(output, "3\n\"2020-01-02\"\n10\n12\ntrue\nfalse\n");
}

#[test]
fn basic_iso_dates_and_times_are_read_by_their_digit_counts() {
    let output = run_example("dates/basic_iso_date_and_time.rb");
    assert_eq!(
        output,
        "{year: 2015, mon: 8, mday: 30, hour: 12, min: 36, sec: 0, zone: \"Z\", offset: 0}\n\
         {year: 2015, mon: 8, mday: 30}\n\
         {year: 2015, mon: 8, mday: 30, hour: 12}\n\
         {year: 123456}\n\
         2015-08-30 12:36:00 UTC\n\
         {year: 2016, mon: 1, mday: 1, hour: 11, min: 22, sec: 33, zone: \"Z\", offset: 0}\n\
         {year: 2016, mon: 1, mday: 1, hour: 11, min: 22, zone: \"+0900\", offset: 32400}\n\
         {year: 2016, mon: 1, mday: 1, hour: 11}\n\
         {mon: 1, mday: 1, hour: 11, min: 22}\n\
         2016-01-01 11:22:33 UTC\n\
         {hour: 12, mday: 30, min: 36, mon: 8, offset: 0, sec: 0, year: 2015, zone: \"Z\"}\n\
         {year: 2015, mon: 8, mday: 30}\n\
         2015-08-30 12:36:00 UTC\n"
    );
}

#[test]
fn an_alias_of_time_new_on_the_singleton_class_takes_its_arguments() {
    let output = run_example("dates/aliased_time_constructors.rb");
    assert_eq!(
        output,
        concat!(
            "2004-11-24 01:04:44 +0000\n",
            "2004\n",
            "567000\n",
            "1970-01-01 00:00:00 UTC\n",
            "#<Date: 2004-11-24 ((2453334j,0s,0n),+0s,2299161j)>\n",
            "#<DateTime: 2004-11-24T01:04:44+00:00 ((2453334j,3884s,0n),+0s,2299161j)>\n",
        )
    );
}

#[test]
fn datetime_writes_the_fraction_digits_asked_for_and_dates_their_japanese_era() {
    let output = run_example("dates/datetime_xmlschema_digits.rb");
    assert_eq!(
        output,
        concat!(
            "\"2005-02-21T10:11:12-05:00\"\n",
            "\"2005-02-21T10:11:12.345-05:00\"\n",
            "\"2005-02-21T10:11:12.345678-05:00\"\n",
            "\"H17.02.21T10:11:12.34-05:00\"\n",
            "\"2005-02-21T10:11:12.3-05:00\"\n",
            "\"2005-02-21T00:00:00+00:00\"\n",
            "\"1872-12-31\"\n",
            "\"M06.01.01\"\n",
            "\"M45.07.29\"\n",
            "\"T01.07.30\"\n",
            "\"T15.12.24\"\n",
            "\"S01.12.25\"\n",
            "\"S64.01.07\"\n",
            "\"H01.01.08\"\n",
            "\"H31.04.30\"\n",
            "\"R01.05.01\"\n",
            "\"R06.03.09\"\n",
        )
    );
}

#[test]
fn time_new_without_a_date_reads_the_clock_rather_than_a_stubbed_now() {
    let output = run_example("dates/time_new_without_a_date_ignores_a_stubbed_now.rb");
    assert_eq!(output, "Time\ntrue\n\"stubbed\"\nTime\n");
}

#[test]
fn datetime_takes_its_arithmetic_from_date_so_a_date_alias_reaches_it() {
    let output = run_example("dates/datetime_arithmetic_comes_from_date.rb");
    assert_eq!(
        output,
        concat!(
            "[Date, Date, Date, Date, Date, Comparable, Date, Date, Date, Date, Date]\n",
            "#<DateTime: 2004-11-24T03:28:44-05:00 ((2453334j,30524s,0n),-18000s,2299161j)>\n",
            "#<DateTime: 2004-11-25T01:04:44-05:00 ((2453335j,21884s,0n),-18000s,2299161j)>\n",
            "#<DateTime: 2004-11-24T00:04:44-05:00 ((2453334j,18284s,0n),-18000s,2299161j)>\n",
            "(1/24)\n",
            "#<Date: 2004-11-24 ((2453334j,8640s,0n),+0s,2299161j)>\n",
            "(4/1)\n",
        )
    );
}

#[test]
fn time_local_hears_the_daylight_flag_only_for_an_hour_the_clock_repeats() {
    let output = run_example("dates/time_local_ten_arguments.rb");
    assert_eq!(
        output,
        concat!(
            "2005-04-22 10:10:10 -0500\n",
            "2005-02-22 10:10:10 -0600\n",
            "2005-10-30 01:30:00 -0500\n",
            "2005-10-30 01:30:00 -0600\n",
            "2005-04-03 03:30:00 -0500\n",
            "2005-06-01 12:00:00 -0500\n",
            "2005-04-22 10:10:10 -0500\n",
        )
    );
}

#[test]
fn a_local_time_the_clock_shows_twice_reads_as_the_later() {
    let output = run_example("dates/time_local_in_the_repeated_hour.rb");
    assert_eq!(
        output,
        concat!(
            "2005-10-30 01:00:00 -0500\n",
            "2005-10-30 01:30:00 -0500\n",
            "2005-10-30 01:00:00 -0400\n",
            "2005-10-30 01:00:00 -0500\n",
            "2005-10-30 02:00:00 -0500\n",
            "2005-04-03 03:30:00 -0400\n",
            "2005-03-27 02:30:00 +0930\n",
            "2005-03-27 02:00:00 +0930\n",
        )
    );
}

#[test]
fn a_time_with_a_zone_object_reads_its_offset_again_after_arithmetic() {
    let output = run_example("dates/time_with_a_zone_object_across_dst.rb");
    assert_eq!(
        output,
        concat!(
            "[2005-10-30 01:00:00 -0400, -14400, EasternZone]\n",
            "[2005-10-30 01:00:00 -0500, -18000]\n",
            "[2005-10-30 01:00:00 -0500, -18000]\n",
            "-14400\n",
            "-18000\n",
            "-14400\n",
        )
    );
}

#[test]
fn dates_read_short_offsets_and_the_iso8601_rfc3339_and_xml_schema_forms() {
    let output = run_example("dates/offsets_and_rfc3339_fields.rb");
    assert_eq!(
        output,
        concat!(
            "(0/1)\n",
            "(-5/24)\n",
            "(3/8)\n",
            "(11/48)\n",
            "(11/48)\n",
            "(0/1)\n",
            "(0/1)\n",
            "(0/1)\n",
            "(-5/24)\n",
            "{year: 1999, mon: 12, mday: 31, hour: 19, min: 0, sec: 0, zone: \"-05:00\", offset: -18000, sec_fraction: (1/8)}\n",
            "{year: 1999, mon: 12, mday: 31, hour: 19, min: 0, sec: 0, zone: \"Z\", offset: 0}\n",
            "{year: 1999, mon: 12, mday: 31, hour: 19, min: 0, sec: 0, zone: \"+01:00\", offset: 3600}\n",
            "{}\n",
            "{year: 1999, mon: 12, mday: 31, hour: 19, min: 0, sec: 0, sec_fraction: (1/2), zone: \"+05:00\", offset: 18000}\n",
            "{hour: 19, min: 0, sec: 0, sec_fraction: (1/4), zone: \"Z\", offset: 0}\n",
            "{mon: 12, mday: 31}\n",
            "{mday: 31, zone: \"+01:00\", offset: 3600}\n",
            "{year: 1999, mon: 12}\n",
            "{}\n",
            "{mday: 31, year: 1999, mon: 12, hour: 19, min: 0, sec: 0, sec_fraction: (1/8), zone: \"-05:00\", offset: -18000}\n",
            "#<DateTime: 1999-12-31T19:00:00+05:00 ((2451544j,50400s,0n),+18000s,2299161j)>\n",
            "#<DateTime: 1999-12-31T19:00:00+05:00 ((2451544j,50400s,500000000n),+18000s,2299161j)>\n",
            "{mday: 31, year: 1999, mon: 12}\n",
            "{mday: 31, year: 1999, mon: 12}\n",
            "{mday: 31, year: 1999, mon: 12, hour: 19, min: 0}\n",
            "{}\n",
            "{mday: 31, year: 1999, mon: 12, hour: 19, min: 0, sec: 0, zone: \"+0500\", offset: 18000}\n",
            "{yday: 365, year: 1999}\n",
            "{cweek: 52, cwday: 5, cwyear: 1999}\n",
            "{mday: 31, mon: 12}\n",
            "{hour: 19, min: 0, sec: 0}\n",
            "{}\n",
            "{mday: 31, year: 1999, mon: 12, hour: 19, min: 0, sec: 0, sec_fraction: (1/2), zone: \"+05\", offset: 18000}\n",
        )
    );
}

#[test]
fn test_dates_time_equality_through_comparable_execution() {
    let output = run_example("dates/time_equality_through_comparable.rb");
    assert_eq!(output, "true\ntrue\ntrue\ntrue\nfalse\nComparable\n");
}

#[test]
fn test_dates_date_private_clock_execution() {
    let output = run_example("dates/date_private_clock.rb");
    assert_eq!(
        output,
        "[:hour, :min, :minute, :sec, :second]\n0\n0\n0\n\"private method 'hour' called for an instance of Date\"\n3\ntrue\n"
    );
}

#[test]
fn test_dates_frozen_date_readers_execution() {
    let output = run_example("dates/frozen_date_readers.rb");
    assert_eq!(
        output,
        "true\n[2004, 2, 29]\n\"2004-02-29\"\n\"2004-02-29 10:20:30\"\n"
    );
}

#[test]
fn test_dates_dates_against_numbers_execution() {
    let output = run_example("dates/dates_against_numbers.rb");
    assert_eq!(
        output,
        "0\n0\n-1\n1\n0\n(19524701/8)\n(324697/8)\n(4881175/2)\n(40587/1)\n"
    );
}

#[test]
fn test_dates_float_seconds_execution() {
    let output = run_example("dates/float_seconds.rb");
    assert_eq!(
        output,
        "(7/10)\n(123456789/1000000000)\n(0/1)\n(0/1)\n(0/1)\n15\n(999983907/1000000000)\ntrue\n#<DateTime: 2001-02-03T04:30:00+00:00 ((2451944j,16200s,0n),+0s,2299161j)>\n[Date::Error, \"invalid fraction\"]\n#<DateTime: 2001-02-03T04:05:30+00:00 ((2451944j,14730s,0n),+0s,2299161j)>\n[Date::Error, \"invalid fraction\"]\n#<DateTime: 2001-02-03T04:05:06+00:00 ((2451944j,14706s,500000000n),+0s,2299161j)>\n[Date::Error, \"invalid date\"]\n[Date::Error, \"invalid date\"]\n#<DateTime: 2001-02-04T00:00:00+00:00 ((2451945j,0s,0n),+0s,2299161j)>\n[TypeError, \"invalid minute (not numeric)\"]\n#<DateTime: 2001-02-03T12:00:00+00:00 ((2451944j,43200s,0n),+0s,2299161j)>\n[Date::Error, \"invalid fraction\"]\n[ArgumentError, \"wrong number of arguments (given 9, expected 0..8)\"]\n#<DateTime: 2001-02-03T12:00:00+00:00 ((2451944j,43200s,0n),+0s,2299161j)>\n#<DateTime: 2001-02-03T06:00:00+00:00 ((2451944j,21600s,0n),+0s,2299161j)>\n#<DateTime: 2001-02-03T04:30:00+00:00 ((2451944j,16200s,0n),+0s,2299161j)>\n"
    );
}

#[test]
fn test_dates_strptime_fields_execution() {
    let output = run_example("dates/strptime_fields.rb");
    assert_eq!(
        output,
        "{year: 2004}\n{year: 2004}\n{year: 1904}\n{wnum0: 12}\n{hour: 15}\n{hour: 15}\n{year: 2001, mon: 2, mday: 3, leftover: \"x\"}\n{sec_fraction: (123/1000)}\n{sec_fraction: (-1/2)}\n{wday: 1}\n{zone: \"+09:00\", offset: 32400}\n{zone: \"Eastern Daylight Time\", offset: -14400}\n{zone: \"foo\", offset: nil}\n{zone: \"+5.5\", offset: 19800}\n{zone: \"UTC+0530\", offset: 19800}\n{mday: 5}\n{year: 2001, wnum1: 5, cwday: 6}\nnil\nnil\n{year: 2001}\nnil\nnil\nnil\n{year: 2001, mon: 2}\n{wday: 2, mon: 2, mday: 3, hour: 4, min: 5, sec: 6, year: 2001}\n{year: 2001, mon: 2, mday: 3, hour: 4, min: 5, sec: 6, zone: \"+07:00\", offset: 25200}\n"
    );
}

#[test]
fn test_dates_dates_from_read_fields_execution() {
    let output = run_example("dates/dates_from_read_fields.rb");
    assert_eq!(
        output,
        "#<Date: 2001-02-03 ((2451944j,0s,0n),+0s,2299161j)>\n#<Date: 2001-02-03 ((2451944j,0s,0n),+0s,2299161j)>\n#<Date: 2001-02-03 ((2451944j,0s,0n),+0s,2299161j)>\n#<Date: 2001-02-03 ((2451944j,0s,0n),+0s,2299161j)>\n#<Date: 2001-02-03 ((2451944j,0s,0n),+0s,2299161j)>\n#<Date: 2001-02-03 ((2451944j,0s,0n),+0s,2299161j)>\n#<Date: 2001-02-03 ((2451944j,0s,0n),+0s,2299161j)>\n#<Date: 2016-08-04 ((2457605j,0s,0n),+0s,2299161j)>\n#<DateTime: 2001-02-03T04:05:06+07:00 ((2451943j,75906s,0n),+25200s,2299161j)>\n#<DateTime: 2001-02-03T16:05:06+00:00 ((2451944j,57906s,0n),+0s,2299161j)>\n#<DateTime: 2001-02-03T04:05:06+07:00 ((2451943j,75906s,0n),+25200s,2299161j)>\n#<DateTime: 2001-02-03T04:05:06+07:00 ((2451943j,75906s,0n),+25200s,2299161j)>\n#<DateTime: 1969-12-31T23:59:59+00:00 ((2440587j,86399s,0n),+0s,2299161j)>\n#<DateTime: 1969-12-31T23:59:59+00:00 ((2440587j,86399s,0n),+0s,2299161j)>\n#<DateTime: 2001-02-03T16:00:00+07:00 ((2451944j,32400s,0n),+25200s,2299161j)>\n#<DateTime: 2016-08-04T06:28:00+05:30 ((2457605j,3480s,0n),+19800s,2299161j)>\n#<DateTime: 2001-02-03T23:59:59+00:00 ((2451944j,86399s,0n),+0s,2299161j)>\n#<DateTime: 2001-02-04T00:00:00+00:00 ((2451945j,0s,0n),+0s,2299161j)>\n#<Date: 2001-02-03 ((2451944j,0s,0n),+0s,2299161j)>\n#<Date: 2001-02-03 ((2451944j,0s,0n),+0s,2299161j)>\n#<DateTime: 2001-02-03T04:05:06+07:00 ((2451943j,75906s,0n),+25200s,2299161j)>\n#<DateTime: 2001-02-03T16:05:06+00:00 ((2451944j,57906s,0n),+0s,2299161j)>\n#<Date: 2001-02-03 ((2451944j,0s,0n),+0s,2299161j)>\n#<DateTime: 2001-02-03T04:05:06+01:00 ((2451944j,11106s,500000000n),+3600s,2299161j)>\n#<DateTime: 2001-02-03T04:05:06+07:00 ((2451943j,75906s,0n),+25200s,2299161j)>\n#<DateTime: 2001-02-03T04:05:06+00:00 ((2451944j,14706s,0n),+0s,2299161j)>\n\"invalid date\"\n\"invalid date\"\n"
    );
}

#[test]
fn test_dates_parse_and_strptime_hashes_execution() {
    let output = run_example("dates/parse_and_strptime_hashes.rb");
    assert_eq!(
        output,
        "{wday: 1, zone: \"GMT-0700\", hour: 0, min: 0, sec: 0, year: 2012, mon: 5, mday: 28, offset: -25200}\n{mon: 90, mday: 0}\n{year: 1999, mon: 12, mday: 31, hour: 12, min: 0, sec: 0, zone: \"-08\", offset: -28800}\n{year: 1999, mon: 12, mday: 31, hour: 12, min: 0, sec: 0, zone: \"-08:00\", offset: -28800}\n{year: 1999, mon: 12, mday: 31, hour: 12, min: 0, sec: 0, zone: \"-08:00:00\", offset: -28800}\n{seconds: (1470272280/1)}\n{seconds: 1470272280, sec_fraction: (123456789/1000000000)}\n"
    );
}

#[test]
fn test_dates_zone_pattern_match_execution() {
    let output = run_example("dates/zone_pattern_match.rb");
    assert_eq!(
        output,
        "\"GMT-0700\"\n\"Eastern Standard Time\"\n\"EST dst\"\n\"-08:00:00\"\nnil\n"
    );
}

#[test]
fn test_dates_parse_fields_execution() {
    let output = run_example("dates/parse_fields.rb");
    assert_eq!(
        output,
        "{wday: 1, zone: \"GMT-0700\", hour: 0, min: 0, sec: 0, year: 2012, mon: 5, mday: 28, offset: -25200}\n{mon: 90, mday: 0}\n{year: 2001, mon: 2, mday: 3}\n{zone: \"+07:00\", hour: 4, min: 5, sec: 6, year: 2001, mon: 2, mday: 3, offset: 25200}\n{year: 2001, mon: 2, mday: 3, hour: 4, min: 5, sec: 6, zone: \"+0700\", offset: 25200}\n{hour: 16, min: 5, sec: 6, year: 2001, mon: 2, mday: 3}\n{wday: 6, zone: \"+0700\", hour: 4, min: 5, sec: 6, year: 2001, mon: 2, mday: 3, offset: 25200}\n{year: 2001, mon: 2, mday: 3}\n{year: -2000, mon: 2, mday: 3}\n{year: 2001, mon: 2, mday: 3}\n{year: 2019, mon: 5, mday: 1}\n{year: 2001, mon: 2, mday: 3}\n{year: 2001, mon: 2, mday: 3}\n{year: 2001, mon: 3, mday: 2}\n{year: 2001, mon: 2, mday: 3}\n{year: 2001, mon: 2, mday: 3}\n{year: 2001}\n{cwyear: 2001, cweek: 5, cwday: 6}\n{cwday: 6}\n{mon: 2, mday: 3}\n{mon: 2, mday: 3}\n{year: 2001, yday: 34}\n{yday: 34}\n{hour: 10, min: 20, sec: 30, sec_fraction: (1/2)}\n{hour: 10, min: 20, sec: 30}\n{hour: 13}\n{hour: 10, min: 0}\n{zone: \"EST dst\", hour: 10, min: 0, offset: -14400}\n{year: 2001, mon: 2, mday: 3, zone: \"EST\", offset: -18000}\n{year: 2001, mon: 2, mday: 3, hour: 4, min: 5, zone: \"9\", offset: 32400}\n{mday: 12}\n{hour: 10, min: 0, mday: 12}\n{wday: 6}\n{}\n{}\n{zone: \"-05\", hour: 19, min: 0, sec: 0, year: 1999, mon: 12, mday: 31, offset: -18000}\n{wday: 4, hour: 14, min: 33, sec: 20, year: 2001, mon: 11, mday: 29}\n{mon: 20, mday: 1}\n{hour: 4, min: 5, sec: 6, sec_fraction: (123456789/1000000000), year: 2001, mon: 2, mday: 3}\n{year: 2010, mon: 1, mday: 15}\n{year: 2040, mon: 1, mday: 1}\n{mon: 1, mday: 2}\n{year: 1999, mon: 1, mday: 1}\n{year: 99, mon: 1, mday: 1}\n\"string length (129) exceeds the limit 128\"\n{}\n"
    );
}

#[test]
fn test_dates_time_argument_ranges_execution() {
    let output = run_example("dates/time_argument_ranges.rb");
    assert_eq!(
        output,
        "[[2026, 90, 0], \"argument out of range\"]\n[[2026, 13, 1], \"mon out of range\"]\n[[2026, 0, 1], \"mon out of range\"]\n[[2026, 12, 0], \"mday out of range\"]\n[[2026, 12, 32], \"argument out of range\"]\n[[2026, 16, 1], \"argument out of range\"]\n[[2026, 15, 1], \"mon out of range\"]\n[[2026, 1, 1, 25], \"hour out of range\"]\n[[2026, 1, 1, 24, 1], \"min out of range\"]\n[[2026, 1, 1, 1, 60], \"min out of range\"]\n[[2026, 1, 1, 1, 1, 61], \"sec out of range\"]\n2026-01-01 01:02:00 +0000\n2026-03-02 00:00:00 +0000\n[[2026, -1, 1], \"argument out of range\"]\n[[2026, 1, -1], \"argument out of range\"]\n[[2026, 90, 0], \"argument out of range\"]\n[[2026, 13, 1], \"mon out of range\"]\n[[2026, 1, 32], \"argument out of range\"]\n"
    );
}

#[test]
fn test_dates_zone_object_dst_execution() {
    let output = run_example("dates/zone_object_dst.rb");
    assert_eq!(
        output,
        "[\"2000-01-01T01:00:00-05:00\", -18000, \"SST\", false, true]\n[\"2000-06-01T01:00:00-04:00\", -14400, \"SDT\", true, true]\n[\"2000-06-01T01:00:00-04:00\", -14400, \"SDT\", true, true]\n[\"2000-05-31T20:00:00-04:00\", -14400, \"SDT\", true, true]\n"
    );
}

#[test]
fn test_dates_repeated_wall_clock_execution() {
    let output = run_example("dates/repeated_wall_clock.rb");
    assert_eq!(
        output,
        "2014-10-25 22:00:00 UTC\n2014-10-25 21:00:00 UTC\n2014-10-25 22:00:00 UTC\n2000-10-29 06:30:00 UTC\n2000-10-29 05:30:00 UTC\n2000-10-29 06:30:00 UTC\n2000-04-02 07:30:00 UTC\n2000-03-25 15:15:00 UTC\n2000-03-25 14:45:00 UTC\n"
    );
}
