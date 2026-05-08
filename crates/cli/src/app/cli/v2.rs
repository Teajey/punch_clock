use std::collections::HashMap;

use clap::{builder::PossibleValue, Arg, ArgMatches, Command};

use crate::app::cli::day::Day;
use crate::plugins;

pub struct Base {
    pub action: Option<Action>,
    pub init: bool,
    pub offset: Option<i32>,
    pub skip_hooks: bool,
}

impl Base {
    pub fn parse() -> Self {
        let matches = build_command().get_matches();
        parse_matches(&matches)
    }
}

#[derive(Clone)]
pub enum DayResolution {
    Hour = 1,
    HalfHour = 2,
    ThirdHour = 3,
    QuarterHour = 4,
    TenMinutes = 6,
    FiveMinutes = 12,
    TwoMinutes = 30,
    Minute = 60,
}

impl DayResolution {
    pub fn as_hour_fraction(&self) -> u16 {
        match self {
            DayResolution::Hour => 1,
            DayResolution::HalfHour => 2,
            DayResolution::ThirdHour => 3,
            DayResolution::QuarterHour => 4,
            DayResolution::TenMinutes => 6,
            DayResolution::FiveMinutes => 12,
            DayResolution::TwoMinutes => 30,
            DayResolution::Minute => 60,
        }
    }
}

pub enum Action {
    In {
        comment: Option<String>,
    },
    Out {
        comment: Option<String>,
    },
    Status,
    Dump,
    Edit,
    Stats {
        day: Option<Day>,
    },
    Undo,
    Day {
        date: Option<Day>,
        resolution: DayResolution,
    },
    Plugin {
        name: String,
        args: Vec<String>,
    },
}

impl DayResolution {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "hour" => Some(Self::Hour),
            "half-hour" => Some(Self::HalfHour),
            "third-hour" => Some(Self::ThirdHour),
            "quarter-hour" => Some(Self::QuarterHour),
            "ten-minutes" => Some(Self::TenMinutes),
            "five-minutes" => Some(Self::FiveMinutes),
            "two-minutes" => Some(Self::TwoMinutes),
            "minute" => Some(Self::Minute),
            _ => None,
        }
    }
}

fn build_command() -> Command {
    let mut cmd = Command::new("punch_clock")
        .author(clap::crate_authors!())
        .version(crate::GIT_REVISION)
        .long_version(crate::LONG_VERSION)
        .about(clap::crate_description!())
        .arg(
            Arg::new("init")
                .long("init")
                .help("Create a `.punch_clock` directory in the current working directory")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("offset")
                .short('o')
                .long("offset")
                .help("Override `punch_clock`'s current UTC offset")
                .value_parser(clap::value_parser!(i32)),
        )
        .arg(
            Arg::new("skip-hooks")
                .long("skip-hooks")
                .help("Run a command without triggering any hooks")
                .action(clap::ArgAction::SetTrue),
        )
        .subcommand(
            Command::new("in").about("Start a session").arg(
                Arg::new("comment")
                    .help("Provide a comment associated with the start of this session")
                    .required(false),
            ),
        )
        .subcommand(
            Command::new("out").about("End the current session").arg(
                Arg::new("comment")
                    .help("Provide a comment associated with the end of this session")
                    .required(false),
            ),
        )
        .subcommand(Command::new("status").about("Check whether you're currently in a session"))
        .subcommand(Command::new("dump").about("Print the record, formatted"))
        .subcommand(Command::new("edit").about("Open the record in your editor, in local time"))
        .subcommand(
            Command::new("stats")
                .about("See some stats about your work hours")
                .arg(
                    Arg::new("day")
                        .help("For a particular day (YYYY-MM-DD)")
                        .required(false),
                ),
        )
        .subcommand(Command::new("undo").about("Remove the latest entry in the record"))
        .subcommand(
            Command::new("day")
                .about("Print visualization of a day's work hours (today by default)")
                .arg(Arg::new("date").help("YYYY-MM-DD").required(false))
                .arg(
                    Arg::new("resolution")
                        .short('r')
                        .long("resolution")
                        .value_parser([
                            PossibleValue::new("hour"),
                            PossibleValue::new("half-hour"),
                            PossibleValue::new("third-hour"),
                            PossibleValue::new("quarter-hour"),
                            PossibleValue::new("ten-minutes"),
                            PossibleValue::new("five-minutes"),
                            PossibleValue::new("two-minutes"),
                            PossibleValue::new("minute"),
                        ])
                        .default_value("hour"),
                ),
        );

    let Some(path_var) = std::env::var("PATH").ok() else {
        return cmd;
    };

    let Some(mut plugins) = plugins::try_iter(&path_var).ok() else {
        return cmd;
    };

    let plugin_map: HashMap<_, _> = plugins.lossy().collect();

    for (name, path) in plugin_map {
        let about = format!("Plugin located at {}", path.display());
        let name: &'static str = Box::leak(name.into_boxed_str());
        let about: &'static str = Box::leak(about.into_boxed_str());
        cmd = cmd.subcommand(
            Command::new(name)
                .about(about)
                .disable_version_flag(true)
                .disable_help_flag(true)
                .disable_help_subcommand(true)
                .arg(
                    Arg::new("args")
                        .allow_hyphen_values(true)
                        .trailing_var_arg(true)
                        .num_args(0..)
                        .value_name("ARGS"),
                ),
        );
    }

    cmd
}

fn parse_matches(matches: &ArgMatches) -> Base {
    let init = matches.get_flag("init");
    let offset = matches.get_one::<i32>("offset").copied();
    let skip_hooks = matches.get_flag("skip-hooks");

    let action = match matches.subcommand() {
        Some(("in", sub)) => Some(Action::In {
            comment: sub.get_one::<String>("comment").cloned(),
        }),
        Some(("out", sub)) => Some(Action::Out {
            comment: sub.get_one::<String>("comment").cloned(),
        }),
        Some(("status", _)) => Some(Action::Status),
        Some(("dump", _)) => Some(Action::Dump),
        Some(("edit", _)) => Some(Action::Edit),
        Some(("stats", sub)) => Some(Action::Stats {
            day: sub.get_one::<Day>("day").cloned(),
        }),
        Some(("undo", _)) => Some(Action::Undo),
        Some(("day", sub)) => Some(Action::Day {
            date: sub.get_one::<Day>("date").cloned(),
            resolution: sub
                .get_one::<String>("resolution")
                .and_then(|s| DayResolution::from_str(s))
                .unwrap_or(DayResolution::Hour),
        }),
        Some((ext, sub)) => {
            if let Some(vals) = sub.get_many::<String>("args") {
                Some(Action::Plugin {
                    name: ext.to_owned(),
                    args: vals.map(ToOwned::to_owned).collect(),
                })
            } else {
                Some(Action::Plugin {
                    name: ext.to_owned(),
                    args: vec![],
                })
            }
        }
        None => None,
    };

    Base {
        action,
        init,
        offset,
        skip_hooks,
    }
}
