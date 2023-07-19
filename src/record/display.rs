use std::ops::RangeInclusive;

use chrono::{DateTime, NaiveDate};
use context::Context;

use super::Record;
use crate::{
    app::context,
    error::Result,
    time::{self, ContextTimeZone, NaiveDateOperations},
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
pub fn paint_datetime_pairs_line<Tz: ContextTimeZone>(
    datetime_pairs: Vec<(DateTime<Tz>, DateTime<Tz>)>,
    range: RangeInclusive<DateTime<Tz>>,
    width: usize,
) -> String {
    let range_start = *range.start();
    let range_end = *range.end();
    let mut buf = vec![false; width];

    for (start, end) in datetime_pairs {
        let start_tween = tween_dates(range_start..=range_end, start);
        let end_tween = tween_dates(range_start..=range_end, end);
        let till_end = end_tween - start_tween;
        let paint_start = (width as f32 * start_tween).round() as usize;
        let paint_end = (width as f32 * (start_tween + till_end)).round() as usize;
        for c in &mut buf[paint_start..paint_end] {
            *c = true;
        }
    }

    buf.into_iter().map(|c| if c { "█" } else { "░" }).collect()
}

pub fn paint_day_range<Tz: ContextTimeZone>(
    ctx: &Context<Tz>,
    record: &Record<Tz>,
    range: RangeInclusive<NaiveDate>,
    width: usize,
) -> Result<()> {
    let range_start = *range.start();
    let range_end = *range.end();
    let total_datetime_pairs = record.clone().try_into_cropped_datetime_pairs(
        ctx,
        range_start.into_day_start(ctx)?,
        range_end.into_day_end(ctx)?,
    )?;
    let total_duration = Record::sum_datetime_pairs(total_datetime_pairs);
    println!(
        "Total time: {} hours, {} minutes",
        total_duration.num_hours(),
        total_duration.num_minutes() % 60
    );
    for day in range_start.iter_days().take_while(|d| d <= &range_end) {
        let day_span = time::day_timespan(ctx, day)?;
        let datetime_pairs = record.clone().try_into_cropped_datetime_pairs(
            ctx,
            *day_span.start(),
            *day_span.end(),
        )?;
        let duration = Record::sum_datetime_pairs(datetime_pairs.clone());
        println!(
            "{} {} {}",
            day.format("%F"),
            paint_datetime_pairs_line(datetime_pairs.clone(), day_span, width),
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

    use super::Record;
    use crate::{app::Context, record::Entry};

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

    fn entry(hour1: u32, min1: u32, hour2: u32, min2: u32) -> Entry<FixedOffset> {
        Entry::try_new(datetime(hour1, min1), datetime(hour2, min2)).unwrap()
    }

    fn line_from_record(record: Record<FixedOffset>, width: usize) -> String {
        let ctx = Context::init(FixedOffset::east_opt(0).unwrap()).unwrap();
        let today_start = datetime(0, 0);
        let today_end = today_start.checked_add_days(chrono::Days::new(1)).unwrap();
        let today_end = today_end
            .checked_sub_signed(chrono::Duration::milliseconds(1))
            .unwrap();

        let datetime_pairs = record
            .try_into_cropped_datetime_pairs(&ctx, today_start, today_end)
            .unwrap();

        super::paint_datetime_pairs_line(datetime_pairs, today_start..=today_end, width)
    }

    #[test]
    fn paint_datetime_pairs_line() {
        let line = line_from_record(
            Record {
                entries: vec![entry(0, 0, 12, 0)],
                current_session: None,
            },
            10,
        );
        assert_eq!("█████░░░░░", line);
    }

    #[test]
    fn paint_datetime_pairs_line_middle() {
        let line = line_from_record(
            Record {
                entries: vec![entry(6, 0, 18, 0)],
                current_session: None,
            },
            10,
        );
        assert_eq!("░░░█████░░", line);
    }

    #[test]
    fn paint_datetime_pairs_line_end() {
        let line = line_from_record(
            Record {
                entries: vec![entry(12, 0, 23, 59)],
                current_session: None,
            },
            10,
        );
        assert_eq!("░░░░░█████", line);
    }

    #[test]
    fn paint_datetime_pairs_line_long() {
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
        assert_eq!("█░█░░██░░░███████░░░░░░█", line);
    }

    #[test]
    fn paint_full() {
        let line = super::paint_datetime_pairs_line(
            vec![(datetime(0, 0), datetime(23, 59))],
            datetime(0, 0)..=datetime(23, 59),
            24,
        );
        assert_eq!("████████████████████████", line);
    }

    #[test]
    fn paint_both_ends() {
        let line = super::paint_datetime_pairs_line(
            vec![
                (datetime(0, 0), datetime(6, 0)),
                (datetime(18, 0), datetime(23, 59)),
            ],
            datetime(0, 0)..=datetime(23, 59),
            24,
        );
        assert_eq!("██████░░░░░░░░░░░░██████", line);
    }
}
