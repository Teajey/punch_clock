use std::fmt::Write;
use std::ops::RangeInclusive;

use chrono::DateTime;

use crate::{
    error::Result,
    record::{display::time_range::printable::Info, Record},
    time::ContextTimeZone,
};

use self::printable::Line;

mod printable {
    use std::fmt::Write;

    use chrono::NaiveDateTime;

    #[derive(Debug, Default, Clone, PartialEq, Eq)]
    pub enum Info {
        #[default]
        Empty,
        SessionSpan,
        SessionStart(NaiveDateTime, Option<String>),
        SessionEnd(NaiveDateTime, Option<String>),
        SessionWhole(NaiveDateTime, NaiveDateTime, Option<String>, Option<String>),
        Multi(u32),
    }

    impl Info {
        fn print(&self, width: u8, bg_toggle: bool) -> Result<String, std::fmt::Error> {
            let mut buf = String::new();
            match self {
                Info::Empty => {
                    for i in 0..width {
                        let parity = i % 2 == 0;
                        let blip = if bg_toggle ^ parity { "░" } else { "▒" };
                        write!(buf, "{blip}")?;
                    }
                }
                _ => {
                    for _ in 0..width {
                        write!(buf, "▓")?;
                    }
                }
            }
            Ok(buf)
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Line {
        pub(super) date: NaiveDateTime,
        pub(super) info: Info,
    }

    impl Line {
        pub(super) fn print(
            &self,
            width: u8,
            bg_toggle: bool,
            date_format: &str,
        ) -> Result<String, std::fmt::Error> {
            let mut buf = String::new();
            write!(
                buf,
                "{} {}",
                self.date.format(date_format),
                self.info.print(width, bg_toggle)?
            )?;
            match &self.info {
                Info::Empty | Info::SessionSpan => (),
                Info::SessionStart(dt, comment) => {
                    write!(buf, " In @ {}", dt.format(date_format))?;
                    if let Some(comment) = comment {
                        write!(buf, " {comment:?}")?;
                    }
                }
                Info::SessionEnd(dt, comment) => {
                    write!(buf, " Out @ {}", dt.format(date_format))?;
                    if let Some(comment) = comment {
                        write!(buf, " {comment:?}")?;
                    }
                }
                Info::SessionWhole(start_dt, end_dt, start_comment, end_comment) => {
                    let star_if = |c: Option<&str>| -> &str {
                        if c.is_some() {
                            "*"
                        } else {
                            ""
                        }
                    };
                    write!(
                        buf,
                        " {}{} -> {}{}",
                        star_if(start_comment.as_deref()),
                        start_dt.format(date_format),
                        star_if(end_comment.as_deref()),
                        end_dt.format(date_format),
                    )?;
                    match (start_comment, end_comment) {
                        (None, Some(comment)) | (Some(comment), None) => {
                            write!(buf, " *{comment:?}")?;
                        }
                        _ => (),
                    }
                }
                Info::Multi(count) => {
                    write!(buf, " [{count} transitions overlapping]")?;
                }
            }
            Ok(buf)
        }
    }
}

pub struct TimeRange(Vec<printable::Line>);

impl TimeRange {
    pub fn print(&self, width: u8, date_format: &str) -> Result<String, std::fmt::Error> {
        let mut buf = String::new();
        for (i, line) in self.0.iter().enumerate() {
            if i != 0 {
                writeln!(buf)?;
            }
            write!(buf, "{}", line.print(width, i % 2 == 0, date_format)?)?;
        }
        Ok(buf)
    }
}

#[allow(clippy::too_many_lines)] // TODO: Yes, I know clippy, this function needs a refactor
pub fn time_range<Tz: ContextTimeZone>(
    record: &Record<Tz>,
    now: DateTime<Tz>,
    range: RangeInclusive<DateTime<Tz>>,
    resolution: u16,
) -> Result<TimeRange> {
    let range_start = *range.start();
    let range_end = *range.end();
    let range_span = range_end - range_start;
    let range_slice = range_span / resolution.into();
    let points = (0..resolution)
        .map(|i| {
            range_start
                + range_slice
                    * i32::try_from(i)
                        .expect("If this value doesn't fit inside i32 what are you even doing.")
        })
        .collect::<Vec<_>>();
    let mut lines: Vec<Line> = points
        .iter()
        .map(|p| Line {
            date: p.naive_local(),
            info: Info::Empty,
        })
        .collect();
    for entry in record.get_entries() {
        let check_out = entry.get_check_out()?;
        if check_out < range_start {
            continue;
        }
        let check_in = entry.check_in;
        if check_in > range_end {
            break;
        }
        let mut first_session_line_printed = false;
        let mut last_session_line = None;
        for (i, line) in lines.iter_mut().enumerate() {
            let point_start = points[i];
            let point_end = point_start + range_slice;
            if point_start >= check_out {
                break;
            }
            if check_in < point_end {
                line.info = match line.info {
                    Info::Empty if first_session_line_printed => Info::SessionSpan,
                    Info::Empty => {
                        Info::SessionStart(check_in.naive_local(), entry.in_comment.clone())
                    }
                    Info::SessionSpan => {
                        unreachable!("There shouldn't ever be more than one session here.")
                    }
                    Info::SessionStart(_, _) | Info::SessionEnd(_, _) => Info::Multi(2),
                    Info::SessionWhole(_, _, _, _) => Info::Multi(3),
                    Info::Multi(count) => Info::Multi(count + 1),
                };
                if !first_session_line_printed && !matches!(line.info, Info::Empty) {
                    first_session_line_printed = true;
                }

                last_session_line = Some(line);
            }
        }
        if let Some(line) = last_session_line {
            match (&mut line.info, entry.out_comment.as_ref()) {
                (Info::SessionSpan, comment) => {
                    line.info = Info::SessionEnd(check_out.naive_local(), comment.cloned());
                }
                (Info::SessionEnd(_, session_comment @ None), comment @ Some(_)) => {
                    *session_comment = comment.cloned();
                }
                (Info::SessionStart(start_dt, start_comment), end_comment) => {
                    line.info = Info::SessionWhole(
                        *start_dt,
                        check_out.naive_local(),
                        start_comment.as_ref().cloned(),
                        end_comment.cloned(),
                    );
                }
                (Info::SessionEnd(_, _) | Info::SessionWhole(_, _, _, _), _) => unreachable!(
                    "Not expecting the end of a session to encounter another complete session"
                ),
                (Info::Multi(count), _) => {
                    *count += 1;
                }
                (Info::Empty, _) => (),
            }
        }
    }

    if let Some((check_in, in_comment)) = &record.current_session {
        let mut first_session_line_printed = false;
        for (i, line) in lines.iter_mut().enumerate() {
            let point_start = points[i];
            let point_end = point_start + range_slice;
            if point_start >= now {
                break;
            }
            if check_in < &point_end {
                line.info = match line.info {
                    Info::Empty if first_session_line_printed => Info::SessionSpan,
                    Info::Empty => Info::SessionStart(check_in.naive_local(), in_comment.clone()),
                    Info::SessionSpan
                    | Info::SessionStart(_, _)
                    | Info::SessionEnd(_, _)
                    | Info::SessionWhole(_, _, _, _)
                    | Info::Multi(_) => {
                        unreachable!("Current session should not encounter another session.")
                    }
                };
                if !first_session_line_printed && !matches!(line.info, Info::Empty) {
                    first_session_line_printed = true;
                }
            }
        }
    }

    Ok(TimeRange(lines))
}

#[cfg(test)]
mod test {
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use pretty_assertions::assert_eq;

    use crate::record::{
        display::time_range::printable::{Info, Line},
        Record,
    };

    /// Zero offset datetime
    macro_rules! dt {
        ($y:literal, $m:literal, $d:literal) => {
            chrono::TimeZone::with_ymd_and_hms(
                &chrono::FixedOffset::east_opt(0).unwrap(),
                $y,
                $m,
                $d,
                0,
                0,
                0,
            )
            .unwrap()
        };
    }

    macro_rules! naive {
        ($y:literal, $m:literal, $d:literal, $h:literal, $mi:literal) => {
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt($y, $m, $d).unwrap(),
                NaiveTime::from_hms_opt($h, $mi, 0).unwrap(),
            )
        };
    }

    macro_rules! s {
        ($s:literal) => {
            $s.to_string()
        };
    }

    #[test]
    fn time_range() {
        let record_file = "2023-01-02T00:00:00.000000+00:00
2023-01-03T00:00:00.000000+00:00 Today was a good day.";
        let record = Record::try_from(record_file).unwrap();
        let tr = super::time_range(
            &record,
            dt!(2024, 1, 1),
            dt!(2023, 1, 1)..=dt!(2023, 1, 4),
            6,
        )
        .unwrap();
        eprintln!("{}", tr.print(4, "%F").unwrap());
        let expected = vec![
            Line {
                date: naive!(2023, 1, 1, 0, 0),
                info: Info::Empty,
            },
            Line {
                date: naive!(2023, 1, 1, 12, 0),
                info: Info::Empty,
            },
            Line {
                date: naive!(2023, 1, 2, 0, 0),
                info: Info::SessionStart(naive!(2023, 1, 2, 0, 0), None),
            },
            Line {
                date: naive!(2023, 1, 2, 12, 0),
                info: Info::SessionEnd(naive!(2023, 1, 3, 0, 0), Some(s!("Today was a good day."))),
            },
            Line {
                date: naive!(2023, 1, 3, 0, 0),
                info: Info::Empty,
            },
            Line {
                date: naive!(2023, 1, 3, 12, 0),
                info: Info::Empty,
            },
        ];
        assert_eq!(expected, tr.0);
    }

    #[test]
    fn time_range_day() {
        let record_file = "2023-11-02T09:30:25.426260+00:00
2023-11-02T11:24:42.221260+00:00

2023-11-02T13:10:22.138841+00:00
2023-11-02T14:34:36.184841+00:00

2023-11-02T14:55:00.061850+00:00
2023-11-02T15:53:38.141850+00:00

2023-11-02T20:47:22.213984+00:00
2023-11-02T22:51:02.408984+00:00
";
        let record = Record::try_from(record_file).unwrap();
        let tr = super::time_range(
            &record,
            dt!(2024, 1, 1),
            dt!(2023, 11, 2)..=dt!(2023, 11, 3),
            24,
        )
        .unwrap();
        insta::assert_display_snapshot!(tr.print(6, "%R").unwrap());
    }

    #[test]
    fn time_range_overlapping_comments() {
        let record_file = "2023-01-01T12:01:00.000000+00:00
2023-01-01T12:02:00.000000+00:00 Good session.

2023-01-01T12:04:00.000000+00:00
2023-01-01T12:05:00.000000+00:00 Even better session!

2023-01-02T12:01:00.000000+00:00
2023-01-02T12:02:00.000000+00:00 Beeble weeble dee 1.

2023-01-02T12:03:00.000000+00:00
2023-01-02T12:04:00.000000+00:00 Beeble weeble dee 2.

2023-01-02T12:05:00.000000+00:00
2023-01-02T12:06:00.000000+00:00 Beeble weeble dee 3.
";
        let record = Record::try_from(record_file).unwrap();
        let tr = super::time_range(
            &record,
            dt!(2024, 1, 1),
            dt!(2023, 1, 1)..=dt!(2023, 1, 4),
            6,
        )
        .unwrap();
        eprintln!("{}", tr.print(6, "%x %X").unwrap());
        let expected = vec![
            Line {
                date: naive!(2023, 1, 1, 0, 0),
                info: Info::Empty,
            },
            Line {
                date: naive!(2023, 1, 1, 12, 0),
                info: Info::Multi(4),
            },
            Line {
                date: naive!(2023, 1, 2, 0, 0),
                info: Info::Empty,
            },
            Line {
                date: naive!(2023, 1, 2, 12, 0),
                info: Info::Multi(6),
            },
            Line {
                date: naive!(2023, 1, 3, 0, 0),
                info: Info::Empty,
            },
            Line {
                date: naive!(2023, 1, 3, 12, 0),
                info: Info::Empty,
            },
        ];
        assert_eq!(expected, tr.0);
    }
}
