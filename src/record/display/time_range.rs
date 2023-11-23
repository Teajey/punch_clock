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

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Comment {
        Single(String),
        Multi(u32),
    }

    #[derive(Debug, Default, Clone, PartialEq, Eq)]
    pub enum Info {
        #[default]
        Empty,
        Session(Option<Comment>),
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
                Info::Session(_) => {
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
                Info::Empty | Info::Session(None) => (),
                Info::Session(Some(Comment::Single(comment))) => {
                    write!(buf, " {comment}")?;
                }
                Info::Session(Some(Comment::Multi(count))) => {
                    write!(buf, " [{count} overlapping comments]")?;
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

pub fn time_range<Tz: ContextTimeZone>(
    record: &Record<Tz>,
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
        // let mut comment_printed = false;
        let mut last_session_line = None;
        for (i, line) in lines.iter_mut().enumerate() {
            let point = points[i];
            if check_in <= point && point < check_out {
                // let comment = match &entry.comment {
                //     Some(comment) if !comment_printed => {
                //         comment_printed = true;
                //         Some(comment.clone())
                //     }
                //     _ => None,
                // };
                line.info = Info::Session(None);
                last_session_line = Some(line);
            }
        }
        match (last_session_line, &entry.comment) {
            (
                Some(Line {
                    date: _,
                    info: Info::Session(session_comment @ None),
                }),
                Some(comment),
            ) => {
                *session_comment = Some(printable::Comment::Single(comment.clone()));
            }
            (
                Some(Line {
                    date: _,
                    info: Info::Session(Some(session_comment @ printable::Comment::Single(_))),
                }),
                Some(_),
            ) => {
                *session_comment = printable::Comment::Multi(2);
            }
            (
                Some(Line {
                    date: _,
                    info: Info::Session(Some(printable::Comment::Multi(count))),
                }),
                Some(_),
            ) => {
                *count += 1;
            }
            (
                Some(Line {
                    date: _,
                    info: Info::Empty,
                }),
                Some(_),
            ) => unreachable!("The last session line must have Info::Session!"),
            _ => (),
        }
    }
    Ok(TimeRange(lines))
}

#[cfg(test)]
mod test {
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use pretty_assertions::assert_eq;

    use crate::record::{
        display::time_range::printable::{Comment, Info, Line},
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
        ($y:literal, $m:literal, $d:literal, $h:literal, $mi:literal, $s:literal) => {
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt($y, $m, $d).unwrap(),
                NaiveTime::from_hms_opt($h, $mi, $s).unwrap(),
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
        let record_file = "2023-01-02T00:00:00.000000+00:00 2023-01-03T00:00:00.000000+00:00 Today was a good day.";
        let record = Record::try_from(record_file).unwrap();
        let tr = super::time_range(&record, dt!(2023, 1, 1)..=dt!(2023, 1, 4), 6).unwrap();
        let expected = vec![
            Line {
                date: naive!(2023, 1, 1, 0, 0, 0),
                info: Info::Empty,
            },
            Line {
                date: naive!(2023, 1, 1, 12, 0, 0),
                info: Info::Empty,
            },
            Line {
                date: naive!(2023, 1, 2, 0, 0, 0),
                info: Info::Session(None),
            },
            Line {
                date: naive!(2023, 1, 2, 12, 0, 0),
                info: Info::Session(Some(Comment::Single(s!("Today was a good day.")))),
            },
            Line {
                date: naive!(2023, 1, 3, 0, 0, 0),
                info: Info::Empty,
            },
            Line {
                date: naive!(2023, 1, 3, 12, 0, 0),
                info: Info::Empty,
            },
        ];
        assert_eq!(expected, tr.0);
        insta::assert_display_snapshot!(tr.print(4, "%F").unwrap());
    }

    #[test]
    fn time_range_day() {
        let record_file = "2023-11-02T09:30:25.426260+00:00 2023-11-02T11:24:42.221260+00:00
2023-11-02T13:10:22.138841+00:00 2023-11-02T14:34:36.184841+00:00
2023-11-02T14:55:00.061850+00:00 2023-11-02T15:53:38.141850+00:00
2023-11-02T20:47:22.213984+00:00 2023-11-02T22:51:02.408984+00:00
";
        let record = Record::try_from(record_file).unwrap();
        let tr = super::time_range(&record, dt!(2023, 11, 2)..=dt!(2023, 11, 3), 24).unwrap();
        insta::assert_display_snapshot!(tr.print(6, "%R").unwrap());
    }

    #[test]
    fn time_range_overlapping_comments() {
        let record_file =
            "2023-01-01T12:01:00.000000+00:00 2023-01-01T12:02:00.000000+00:00 Good session.
2023-01-01T12:04:00.000000+00:00 2023-01-01T12:05:00.000000+00:00 Even better session!
2023-01-02T12:01:00.000000+00:00 2023-01-02T12:02:00.000000+00:00 Beeble weeble dee 1.
2023-01-02T12:03:00.000000+00:00 2023-01-02T12:04:00.000000+00:00 Beeble weeble dee 2.
2023-01-02T12:05:00.000000+00:00 2023-01-02T12:06:00.000000+00:00 Beeble weeble dee 3.
";
        let record = Record::try_from(record_file).unwrap();
        let tr = super::time_range(&record, dt!(2023, 1, 1)..=dt!(2023, 1, 4), 6).unwrap();
        let expected = vec![
            Line {
                date: naive!(2023, 1, 1, 0, 0, 0),
                info: Info::Empty,
            },
            Line {
                date: naive!(2023, 1, 1, 12, 0, 0),
                info: Info::Session(Some(Comment::Multi(2))),
            },
            Line {
                date: naive!(2023, 1, 2, 0, 0, 0),
                info: Info::Session(None),
            },
            Line {
                date: naive!(2023, 1, 2, 12, 0, 0),
                info: Info::Session(Some(Comment::Multi(3))),
            },
            Line {
                date: naive!(2023, 1, 3, 0, 0, 0),
                info: Info::Empty,
            },
            Line {
                date: naive!(2023, 1, 3, 12, 0, 0),
                info: Info::Empty,
            },
        ];
        assert_eq!(expected, tr.0);
    }
}
