use std::fmt::Display;
use std::str::FromStr;
use std::path::Path;

pub fn arg_err(idx: usize, str: &str, message: impl Display) {
    println!("Error in arg {} \"{}\": {}", idx + 1, str, message)
}

pub trait ArgParser<'a, T> {
    /// Possible names, including any "-"
    const NAMES: &'static [&'static str];
    /// Possible values, only used for help
    const VALUES: &'static [&'static str];
    const DESCRIPTION: &'static str;
    fn parse_arg_value(value: Option<&'a str>) -> Result<T, String>;
    fn try_parse(input: &'a[String], used: &mut [bool]) -> Option<T> {
        input.iter().enumerate().filter_map(|(idx, it)| {
            for name in Self::NAMES {
                if it.starts_with(name) {
                    let rest = &it[name.len()..];
                    return if rest.is_empty() {
                        Some((idx, it, Self::parse_arg_value(None).map_err(|err| arg_err(idx, it, err))))
                    } else if rest.starts_with("=") {
                        Some((idx, it, Self::parse_arg_value(Some(&rest[1..])).map_err(|err|arg_err(idx, it, err))))
                    } else {
                        None
                    };
                }
            }
            None
        }).map(|it| {
            used[it.0] = true;
            it
        }).fold(None, |value, (idx, it, res)| {
            match res {
                Ok(ok) => {
                    if value.is_none() {
                        Some(ok)
                    } else {
                        arg_err(idx, it, "Arg has already been set, ignoring");
                        value
                    }
                }
                Err(()) => {
                    value
                }
            }
        })
    }
}

pub trait DefaultArgParser<'a, T>: ArgParser<'a, T> {

    const DEFAULT: T;

    fn parse(input: &'a[String], used: &mut [bool]) -> T {
        let value = Self::try_parse(input, used);
        value.unwrap_or(Self::DEFAULT)
    }
}

pub trait BoolParser<'a>: ArgParser<'a, bool> {
    const NAMES: &'static [&'static str];
    const DESCRIPTION: &'static str;
}

impl<'a, T: BoolParser<'a>> ArgParser<'a, bool> for T {
    const NAMES: &'static [&'static str] = <T as BoolParser>::NAMES;
    const VALUES: &'static [&'static str] = &["true", "t", "y", "false", "f", "n"];
    const DESCRIPTION: &'static str = <T as BoolParser>::DESCRIPTION;

    fn parse_arg_value(value: Option<&str>) -> Result<bool, String> {
        if let Some(value) = value {
            match value {
                "true" | "t" | "y" => Ok(true),
                "false" | "f" | "n" => Ok(false),
                _ => Err(format!("Invalid value \"{}\", must be one of {}", value, Self::VALUES.join(", ")))
            }
        } else {
            Ok(true)
        }
    }
}

impl<'a, T: BoolParser<'a>> DefaultArgParser<'a, bool> for T {
    const DEFAULT: bool = false;
}

pub trait F64Parser<'a>: ArgParser<'a, f64> {
    const NAMES: &'static [&'static str];
    const DESCRIPTION: &'static str;
}

impl<'a, T: F64Parser<'a>> ArgParser<'a, f64> for T {
    const NAMES: &'static [&'static str] = <T as F64Parser>::NAMES;
    const VALUES: &'static [&'static str] = &["<float>", "<double>", "<int>"];
    const DESCRIPTION: &'static str = <T as F64Parser>::DESCRIPTION;

    fn parse_arg_value(value: Option<&str>) -> Result<f64, String> {
        if let Some(value) = value {
            match f64::from_str(value) {
                Ok(value) => Ok(value),
                Err(err) => {
                    Err(format!("\"{}\": {}", value, err))
                }
            }
        } else {
            Err("Arg value is not optional, --help for more info".into())
        }
    }
}

pub trait StringParser<'a>: ArgParser<'a, &'a str> {
    const NAMES: &'static [&'static str];
    const DESCRIPTION: &'static str;
}

impl<'a, T: StringParser<'a>> ArgParser<'a, &'a str> for T {
    const NAMES: &'static [&'static str] = <T as StringParser>::NAMES;
    const VALUES: &'static [&'static str] = &["<string>"];
    const DESCRIPTION: &'static str = <T as StringParser>::DESCRIPTION;

    fn parse_arg_value(value: Option<&'a str>) -> Result<&'a str, String> {
        if let Some(value) = value {
            Ok(value)
        } else {
            Err("Arg value is not optional, --help for more info".into())
        }
    }
}

pub trait FileParser<'a>: ArgParser<'a, &'a Path> {
    const NAMES: &'static [&'static str];
    const DESCRIPTION: &'static str;
}

impl<'a, T: FileParser<'a>> ArgParser<'a, &'a Path> for T {
    const NAMES: &'static [&'static str] = <T as FileParser>::NAMES;
    const VALUES: &'static [&'static str] = &["<path>"];
    const DESCRIPTION: &'static str = <T as FileParser>::DESCRIPTION;

    fn parse_arg_value(value: Option<&'a str>) -> Result<&'a Path, String> {
        if let Some(value) = value {
            Ok(Path::new(value))
        } else {
            Err("Arg value is not optional, --help for more info".into())
        }
    }
}
