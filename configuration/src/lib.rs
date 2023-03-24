//! Configuration parsing crate.
//! Example configuration:
//! ```ignore
//! include /etc/passwd
//! include /home/user
//! exclude /home/user/.local
//! ```

use std::path::PathBuf;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{multispace0, multispace1},
    multi::separated_list1,
    sequence::{delimited, tuple},
    IResult,
};

#[derive(Default)]
struct Config {
    includes: Vec<PathBuf>,
    excludes: Vec<PathBuf>,
}

#[derive(Debug, PartialEq)]
enum ConfigLine {
    Include(Vec<PathBuf>),
    Exclude(Vec<PathBuf>),
}

fn parse_config_line(input: &str) -> IResult<&str, ConfigLine, ()> {
    alt((include_line, exclude_line))(input)
}

fn include_line(input: &str) -> IResult<&str, ConfigLine, ()> {
    let (tail, (_, _, paths)) = tuple((tag("include"), multispace1, path_list))(input)?;
    Ok((tail, ConfigLine::Include(paths)))
}

fn exclude_line(input: &str) -> IResult<&str, ConfigLine, ()> {
    let (tail, (_, _, paths)) = tuple((tag("exclude"), multispace1, path_list))(input)?;
    Ok((tail, ConfigLine::Exclude(paths)))
}

fn path_list(input: &str) -> IResult<&str, Vec<PathBuf>, ()> {
    let (tail, paths) = separated_list1(
        delimited(multispace0, tag(","), multispace0),
        take_while(|c| c != ',' && c != '\n'),
    )(input)?;
    Ok((
        tail,
        paths.iter().map(|p| p.trim().parse().unwrap()).collect(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_config_lines() {
        let test_cases = vec![
            (
                "include /etc/path",
                ConfigLine::Include(vec![PathBuf::from("/etc/path")]),
            ),
            (
                "include\t/etc/a,/etc/b",
                ConfigLine::Include(vec![PathBuf::from("/etc/a"), PathBuf::from("/etc/b")]),
            ),
            (
                "exclude /etc/a",
                ConfigLine::Exclude(vec![PathBuf::from("/etc/a")]),
            ),
        ];

        for test_case in test_cases {
            let (_, actual) = parse_config_line(test_case.0).unwrap();
            assert_eq!(test_case.1, actual);
        }
    }

    #[test]
    fn parses_file_lists() {
        let test_cases = vec![
            ("/etc/file", vec!["/etc/file"]),
            ("/etc/a,/etc/b,/etc/c", vec!["/etc/a", "/etc/b", "/etc/c"]),
            (
                " /etc/a  , /etc/b   ,  /etc/c ",
                vec!["/etc/a", "/etc/b", "/etc/c"],
            ),
            (
                "\t/etc/a\t,/etc/b,/etc/c",
                vec!["/etc/a", "/etc/b", "/etc/c"],
            ),
        ];

        for test_case in test_cases {
            let (_, actual) = path_list(test_case.0).unwrap();
            assert_eq!(
                test_case
                    .1
                    .iter()
                    .map(|p| p.parse().unwrap())
                    .collect::<Vec<PathBuf>>(),
                actual
            );
        }
    }
}
