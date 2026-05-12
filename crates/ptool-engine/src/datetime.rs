use crate::{Error, ErrorKind, Result};
use jiff::{Timestamp, Zoned, civil::DateTime as CivilDateTime, fmt::strtime, tz::TimeZone};
use std::cmp::Ordering;
use std::fmt;

const NOW_OP: &str = "ptool.datetime.now";
const PARSE_OP: &str = "ptool.datetime.parse";
const FROM_UNIX_OP: &str = "ptool.datetime.from_unix";
const FORMAT_OP: &str = "ptool.datetime.DateTime:format";
const IN_TZ_OP: &str = "ptool.datetime.DateTime:in_tz";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DateTimeUnixUnit {
    #[default]
    Second,
    Millisecond,
    Nanosecond,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DateTimeParseOptions {
    pub timezone: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DateTimeFromUnixOptions {
    pub unit: DateTimeUnixUnit,
    pub timezone: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DateTimeValue {
    zoned: Zoned,
}

impl DateTimeValue {
    pub fn year(&self) -> i16 {
        self.zoned.year()
    }

    pub fn month(&self) -> i8 {
        self.zoned.month()
    }

    pub fn day(&self) -> i8 {
        self.zoned.day()
    }

    pub fn hour(&self) -> i8 {
        self.zoned.hour()
    }

    pub fn minute(&self) -> i8 {
        self.zoned.minute()
    }

    pub fn second(&self) -> i8 {
        self.zoned.second()
    }

    pub fn nanosecond(&self) -> i32 {
        self.zoned.subsec_nanosecond()
    }

    pub fn offset(&self) -> String {
        format_offset(self.zoned.offset().seconds())
    }

    pub fn timezone(&self) -> String {
        if let Some(name) = self.zoned.time_zone().iana_name() {
            return name.to_string();
        }
        if self.zoned.time_zone().is_unknown() {
            return "Etc/Unknown".to_string();
        }
        self.offset()
    }

    pub fn format(&self, format: &str) -> Result<String> {
        strtime::format(format, &self.zoned).map_err(|err| invalid_datetime(FORMAT_OP, format, err))
    }

    pub fn unix(&self, unit: DateTimeUnixUnit) -> i128 {
        match unit {
            DateTimeUnixUnit::Second => i128::from(self.zoned.timestamp().as_second()),
            DateTimeUnixUnit::Millisecond => i128::from(self.zoned.timestamp().as_millisecond()),
            DateTimeUnixUnit::Nanosecond => self.zoned.timestamp().as_nanosecond(),
        }
    }

    pub fn in_tz(&self, timezone: &str) -> Result<DateTimeValue> {
        let zoned = self
            .zoned
            .in_tz(timezone)
            .map_err(|err| invalid_datetime(IN_TZ_OP, timezone, err))?;
        Ok(DateTimeValue { zoned })
    }
}

pub(crate) fn now(timezone: Option<&str>) -> Result<DateTimeValue> {
    let zoned = match timezone {
        Some(timezone) => Zoned::now()
            .in_tz(timezone)
            .map_err(|err| invalid_datetime(NOW_OP, timezone, err))?,
        None => Zoned::now(),
    };
    Ok(DateTimeValue { zoned })
}

pub(crate) fn parse(input: &str, options: DateTimeParseOptions) -> Result<DateTimeValue> {
    if input.is_empty() {
        return Err(Error::new(
            ErrorKind::EmptyInput,
            format!("{PARSE_OP} does not accept empty string"),
        )
        .with_op(PARSE_OP)
        .with_input(input));
    }

    if let Some(timezone) = options.timezone.as_deref() {
        if parse_zoned_input(input).is_ok() {
            return Err(Error::new(
                ErrorKind::InvalidArgs,
                format!(
                    "{PARSE_OP} `timezone` must not be set when input already includes a timezone or offset"
                ),
            )
            .with_op(PARSE_OP)
            .with_input(input)
            .with_detail("`timezone` is only for naive datetime input"));
        }

        let datetime = input
            .parse::<CivilDateTime>()
            .map_err(|err| invalid_datetime(PARSE_OP, input, err))?;
        let zoned = datetime
            .in_tz(timezone)
            .map_err(|err| invalid_datetime(PARSE_OP, input, err))?;
        return Ok(DateTimeValue { zoned });
    }

    let zoned = parse_zoned_input(input)?;
    Ok(DateTimeValue { zoned })
}

pub(crate) fn from_unix(value: i64, options: DateTimeFromUnixOptions) -> Result<DateTimeValue> {
    let timestamp = match options.unit {
        DateTimeUnixUnit::Second => Timestamp::from_second(value)
            .map_err(|err| invalid_datetime(FROM_UNIX_OP, value, err))?,
        DateTimeUnixUnit::Millisecond => Timestamp::from_millisecond(value)
            .map_err(|err| invalid_datetime(FROM_UNIX_OP, value, err))?,
        DateTimeUnixUnit::Nanosecond => Timestamp::from_nanosecond(i128::from(value))
            .map_err(|err| invalid_datetime(FROM_UNIX_OP, value, err))?,
    };
    let zoned = match options.timezone.as_deref() {
        Some(timezone) => timestamp
            .in_tz(timezone)
            .map_err(|err| invalid_datetime(FROM_UNIX_OP, value, err))?,
        None => timestamp.to_zoned(TimeZone::UTC),
    };
    Ok(DateTimeValue { zoned })
}

pub(crate) fn compare(a: &DateTimeValue, b: &DateTimeValue) -> Ordering {
    a.cmp(b)
}

impl fmt::Display for DateTimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.zoned
                .timestamp()
                .display_with_offset(self.zoned.offset())
        )
    }
}

fn invalid_datetime(op: &str, input: impl ToString, err: impl std::fmt::Display) -> Error {
    Error::new(
        ErrorKind::InvalidArgs,
        format!("{op} invalid datetime `{}`: {err}", input.to_string()),
    )
    .with_op(op)
    .with_input(input.to_string())
}

fn parse_zoned_input(input: &str) -> Result<Zoned> {
    if let Ok(zoned) = input.parse::<Zoned>() {
        return Ok(zoned);
    }

    for format in [
        "%Y-%m-%dT%H:%M:%S%.f%:z",
        "%Y-%m-%d %H:%M:%S%.f%:z",
        "%Y-%m-%dT%H:%M%:z",
        "%Y-%m-%d %H:%M%:z",
    ] {
        if let Ok(broken_down) = strtime::parse(format, input)
            && let Ok(zoned) = broken_down.to_zoned()
        {
            return Ok(zoned);
        }
    }

    Err(invalid_datetime(
        PARSE_OP,
        input,
        "expected a timezone annotation like `[America/New_York]` or a numeric offset like `-04:00`",
    ))
}

fn format_offset(seconds: i32) -> String {
    let sign = if seconds < 0 { '-' } else { '+' };
    let seconds = seconds.abs();
    let hours = seconds / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let seconds = seconds % 60;

    if seconds == 0 {
        format!("{sign}{hours:02}:{minutes:02}")
    } else {
        format!("{sign}{hours:02}:{minutes:02}:{seconds:02}")
    }
}
