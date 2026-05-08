use std::ops::RangeInclusive;

use chrono::{DateTime, Duration, NaiveDate};
use context::Context;

use punch_clock_core::{
    context,
    error::Result,
    record::Record,
    time::{self, ContextTimeZone, NaiveDateOperations, range::DateTimeRange},
};

#[allow(clippy::cast_precision_loss)]
fn tween_dates<Tz: ContextTimeZone>(range: RangeInclusive<DateTime<Tz>>, pos: DateTime<Tz>) -> f32 {
    assert!(range.contains(&pos), "Pos date provided outside range");
    let start = *range.start();
    let end = *range.end();
    let time_till_pos = pos.signed_duration_since(start).num_milliseconds() as f32;
    let time_till_end = end.signed_duration_since(start).num_milliseconds() as f32;
    time_till_pos / time_till_end
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn paint_datetime_ranges_line<Tz: ContextTimeZone>(
    datetime_ranges: Vec<DateTimeRange<Tz>>,
    range: RangeInclusive<DateTime<Tz>>,
    width: usize,
    background_shift: bool,
) -> String {
    let range_start = *range.start();
    let range_end = *range.end();
    let mut buf = vec![false; width];

    for dtr in datetime_ranges {
        let start_tween = tween_dates(range_start..=range_end, *dtr.start());
        let end_tween = tween_dates(range_start..=range_end, *dtr.end());
        let till_end = end_tween - start_tween;
        let paint_start = (width as f32 * start_tween).round() as usize;
        let paint_end = (width as f32 * (start_tween + till_end)).round() as usize;
        for c in &mut buf[paint_start..paint_end] {
            *c = true;
        }
    }

    buf.into_iter()
        .enumerate()
        .map(|(i, c)| {
            let c_parity = i % 2 != 0;
            if c {
                "▓"
            } else if c_parity ^ background_shift {
                "░"
            } else {
                "▒"
            }
        })
        .collect()
}

pub fn paint_day_range<Tz: ContextTimeZone>(
    ctx: &Context<Tz>,
    record: &Record<Tz>,
    range: RangeInclusive<NaiveDate>,
    width: usize,
) -> Result<()> {
    let range_start = *range.start();
    let range_end = *range.end();
    let total_datetime_ranges = record.clone().try_into_cropped_datetime_ranges(
        ctx,
        range_start.into_day_start(ctx)?,
        range_end.into_day_end(ctx)?,
    )?;
    let days_covered = total_datetime_ranges
        .iter()
        .flat_map(DateTimeRange::days_covered)
        .collect::<std::collections::BTreeSet<_>>()
        .len();

    let total_duration: chrono::Duration = total_datetime_ranges.into_iter().sum();
    println!(
        "Total time: {} hours, {} minutes",
        total_duration.num_hours(),
        total_duration.num_minutes() % 60
    );

    match i32::try_from(days_covered) {
        Ok(0) => {
            println!("# of work days: 0");
            println!("Average work day time: 0 hours, 0 minutes");
        }
        Ok(days_covered) => {
            println!("# of work days: {days_covered}");
            let average_duration = total_duration / days_covered;
            println!(
                "Average work day time: {} hours, {} minutes",
                average_duration.num_hours(),
                average_duration.num_minutes() % 60
            );
        }
        Err(err) => {
            println!("# of work days: FAILED TO PARSE FROM usize: {err}");
            println!("Average work day time: UNAVAILABLE");
        }
    }

    for (i, day) in range_start
        .iter_days()
        .take_while(|d| d <= &range_end)
        .enumerate()
    {
        let day_span = time::day_timespan(ctx, day)?;
        let datetime_ranges = record.clone().try_into_cropped_datetime_ranges(
            ctx,
            *day_span.start(),
            *day_span.end(),
        )?;
        let duration: Duration = datetime_ranges.clone().into_iter().sum();
        println!(
            "{} {} {}",
            day.format("%F"),
            paint_datetime_ranges_line(datetime_ranges, day_span, width, i % 2 != 0),
            if duration.is_zero() {
                String::new()
            } else {
                time::human_readable_duration(&duration)?
            }
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};

    use punch_clock_core::context::{self, Context};
    use punch_clock_core::record::Entry;
    use punch_clock_core::time::range::DateTimeRange;

    use crate::display::paint_day_range;

    use super::Record;

    fn tz() -> FixedOffset {
        FixedOffset::east_opt(0).unwrap()
    }

    fn datetime(hour: u32, min: u32) -> DateTime<FixedOffset> {
        let chrono::LocalResult::Single(dt) = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2023, 7, 15).unwrap(),
            NaiveTime::from_hms_opt(hour, min, 0).unwrap(),
        )
        .and_local_timezone(tz()) else {
            panic!("datetime failed!");
        };

        dt
    }

    fn date_md(month: u32, day: u32) -> chrono::NaiveDate {
        chrono::NaiveDate::from_ymd_opt(2023, month, day).unwrap()
    }

    fn entry(hour1: u32, min1: u32, hour2: u32, min2: u32) -> Entry<FixedOffset> {
        Entry::try_new(datetime(hour1, min1), datetime(hour2, min2), None, None).unwrap()
    }

    fn line_from_record(record: Record<FixedOffset>, width: usize) -> String {
        let ctx = Context::init_with_offset(Default::default(), 0).unwrap();
        let today_start = datetime(0, 0);
        let today_end = today_start.checked_add_days(chrono::Days::new(1)).unwrap();
        let today_end = today_end
            .checked_sub_signed(chrono::Duration::milliseconds(1))
            .unwrap();

        let datetime_ranges = record
            .try_into_cropped_datetime_ranges(&ctx, today_start, today_end)
            .unwrap();

        super::paint_datetime_ranges_line(datetime_ranges, today_start..=today_end, width, false)
    }

    #[test]
    fn paint_datetime_ranges_line() {
        let line = line_from_record(
            Record {
                entries: vec![entry(0, 0, 12, 0)],
                current_session: None,
            },
            10,
        );
        assert_eq!("▓▓▓▓▓░▒░▒░", line);
    }

    #[test]
    fn paint_datetime_ranges_line_middle() {
        let line = line_from_record(
            Record {
                entries: vec![entry(6, 0, 18, 0)],
                current_session: None,
            },
            10,
        );
        assert_eq!("▒░▒▓▓▓▓▓▒░", line);
    }

    #[test]
    fn paint_datetime_ranges_line_end() {
        let line = line_from_record(
            Record {
                entries: vec![entry(12, 0, 23, 59)],
                current_session: None,
            },
            10,
        );
        assert_eq!("▒░▒░▒▓▓▓▓▓", line);
    }

    #[test]
    fn paint_datetime_ranges_line_long() {
        let line = line_from_record(
            Record {
                entries: vec![
                    entry(0, 0, 1, 0),
                    entry(2, 0, 3, 0),
                    entry(5, 0, 7, 0),
                    entry(10, 0, 16, 30),
                    entry(19, 0, 19, 10),
                    entry(23, 0, 23, 59),
                ],
                current_session: None,
            },
            24,
        );
        assert_eq!("▓░▓░▒▓▓░▒░▓▓▓▓▓▓▓░▒░▒░▒▓", line);
    }

    #[test]
    fn paint_full() {
        let line = super::paint_datetime_ranges_line(
            vec![DateTimeRange::new(datetime(0, 0), datetime(23, 59)).unwrap()],
            datetime(0, 0)..=datetime(23, 59),
            24,
            false,
        );
        assert_eq!("▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓", line);
    }

    #[test]
    fn paint_both_ends() {
        let line = super::paint_datetime_ranges_line(
            vec![
                DateTimeRange::new(datetime(0, 0), datetime(6, 0)).unwrap(),
                DateTimeRange::new(datetime(18, 0), datetime(23, 59)).unwrap(),
            ],
            datetime(0, 0)..=datetime(23, 59),
            24,
            false,
        );
        assert_eq!("▓▓▓▓▓▓▒░▒░▒░▒░▒░▒░▓▓▓▓▓▓", line);
    }

    #[test]
    fn range_end_index_x_out_of_range_for_slice_of_length_y() {
        let ctx = context::Context {
            editor_path: String::new(),
            offset: Some(0),
            timezone: FixedOffset::east_opt(0).unwrap(),
            skip_hooks: Default::default(),
        };
        let rec_file = "2023-07-10T05:05:42.372091+00:00 2023-07-10T09:38:44.320091+00:00
2023-07-10T20:00:00+00:00        2023-07-10T22:13:34.369+00:00";
        let rec = Record::try_from(rec_file)
            .unwrap()
            .with_timezone(&ctx.timezone);
        paint_day_range(&ctx, &rec, date_md(7, 9)..=date_md(7, 10), 48).unwrap();
    }

    #[test]
    fn range_end_index_x_out_of_range_for_slice_of_length_y_2() {
        let ctx = context::Context {
            editor_path: String::new(),
            offset: Some(0),
            timezone: FixedOffset::east_opt(0).unwrap(),
            skip_hooks: Default::default(),
        };
        let rec_file = "2023-06-04T21:08:34.790590+00:00 2023-06-04T22:32:47.660590+00:00
2023-06-05T04:30:04.199633+00:00 2023-06-05T07:18:50.734633+00:00";
        let rec = Record::try_from(rec_file)
            .unwrap()
            .with_timezone(&ctx.timezone);
        paint_day_range(&ctx, &rec, date_md(6, 4)..=date_md(6, 5), 48).unwrap();
    }

    #[test]
    fn range_end_index_x_out_of_range_for_slice_of_length_y_3() {
        let ctx = context::Context {
            editor_path: String::new(),
            offset: Some(12 * 3600),
            timezone: FixedOffset::east_opt(12 * 3600).unwrap(),
            skip_hooks: Default::default(),
        };
        let rec_file = "2023-06-30T04:30:00.893153+00:00
2023-06-30T07:15:07.931153+00:00

2023-07-10T05:05:42.372091+00:00
2023-07-10T09:38:44.320091+00:00

2023-07-10T20:00:00+00:00       
2023-07-10T22:13:34.369+00:00   

2023-07-11T04:30:55.569838+00:00
2023-07-11T05:05:55.569838+00:00

2023-07-11T09:01:20.726248+00:00
2023-07-11T12:08:36.149248+00:00

2023-07-11T12:32:28.616529+00:00
2023-07-11T14:27:00.836529+00:00

2023-07-11T20:03:53.114039+00:00
2023-07-11T22:41:25.885039+00:00

2023-07-12T04:30:00+00:00       
2023-07-12T04:54:25.885+00:00   

2023-07-12T09:30:00+00:00       
2023-07-12T13:00:00+00:00       

2023-07-12T22:04:34.947469+00:00
2023-07-12T23:29:44.706469+00:00

2023-07-13T09:08:38.290767+00:00
2023-07-13T10:34:50.199767+00:00
";
        let rec = Record::try_from(rec_file)
            .unwrap()
            .with_timezone(&ctx.timezone);
        paint_day_range(&ctx, &rec, date_md(7, 10)..=date_md(7, 12), 24).unwrap();
    }
}
