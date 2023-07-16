use chrono::NaiveDate;

#[derive(Clone)]
pub struct Day(pub NaiveDate);

impl clap::builder::ValueParserFactory for Day {
    type Parser = Parser;
    fn value_parser() -> Self::Parser {
        Parser
    }
}

#[derive(Clone, Debug)]
pub struct Parser;

impl clap::builder::TypedValueParser for Parser {
    type Value = Day;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let inner = clap::builder::NonEmptyStringValueParser::new();
        let val = inner.parse_ref(cmd, arg, value)?;

        if let Ok(date) = NaiveDate::parse_from_str(&val, "%F") {
            return Ok(Day(date));
        }

        let mut err = clap::Error::new(clap::error::ErrorKind::ValueValidation).with_cmd(cmd);

        if let Some(arg) = arg {
            err.insert(
                clap::error::ContextKind::InvalidArg,
                clap::error::ContextValue::String(arg.to_string()),
            );
            err.insert(
                clap::error::ContextKind::InvalidValue,
                clap::error::ContextValue::String(val),
            );
        }

        // TODO: Bother clap-rs devs about this
        // err.inner.set_source()

        Err(err)
    }
}
