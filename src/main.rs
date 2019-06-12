// Many incorrect assumptions were made when creating this initially.
// See the following for a better description on the format:
// https://www.cyberciti.biz/faq/create-ssh-config-file-on-linux-unix/
// https://linux.die.net/man/5/ssh_config

use nom::{
    branch::alt,
    bytes::streaming::{tag, take_while1},
    character::streaming::{line_ending, not_line_ending, space0, space1},
    combinator::{complete, peek},
    multi::{many0, many_till},
    sequence::tuple,
    IResult,
};
use std::{fs, path::PathBuf};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opts {
    pub input: PathBuf,
}

fn main() {
    let opt = Opts::from_args();
    let data = fs::read_to_string(&opt.input).unwrap_or_else(|_| {
        println!("Could not open file '{}'", &opt.input.display());
        std::process::exit(1);
    });

    println!("{:#?}", hosts(&data));
}

#[derive(Debug)]
struct Host<'a> {
    name: &'a str,
    properties: Vec<Property<'a>>,
}

#[derive(Debug)]
struct Property<'a> {
    key: &'a str,
    value: &'a str,
}

fn string(i: &str) -> IResult<&str, &str> {
    take_while1(|c: char| !c.is_whitespace())(i)
}

fn complete_string(i: &str) -> IResult<&str, &str> {
    let res = complete(string)(i);

    if let Err(nom::Err::Error((value, nom::error::ErrorKind::Complete))) = res {
        if !value.is_empty() {
            Ok(("", value))
        } else {
            res
        }
    } else {
        res
    }
}

fn complete_space0(i: &str) -> IResult<&str, &str> {
    let res = space0(i);

    if let Err(nom::Err::Incomplete(nom::Needed::Size(1))) = res {
        Ok(("", ""))
    } else {
        res
    }
}

fn complete_line_ending(i: &str) -> IResult<&str, &str> {
    let res = line_ending(i);

    if let Err(nom::Err::Incomplete(nom::Needed::Size(1))) = res {
        Ok(("", ""))
    } else {
        res
    }
}

fn host_line(i: &str) -> IResult<&str, &str> {
    let host = tag("Host");
    let parser = tuple((
        space0,
        host,
        space1,
        complete_string,
        complete_space0,
        complete_line_ending,
    ));

    let (input, (_, _, _, name, _, _)) = parser(i)?;

    Ok((input, name))
}

// TODO: Property should become something like
// Property { key, tokens }
// or
// Property { key, values }
// to handle the case where a key has multple values, like for `LocalForward`
fn property_line(i: &str) -> IResult<&str, Property> {
    let parser = tuple((space0, string, space1, not_line_ending, complete_line_ending));
    let (input, (_, key, _, value, _)) = parser(i)?;

    Ok((input, Property { key, value }))
}

fn properties(i: &str) -> IResult<&str, (Vec<Property>, &str)> {
    let parser = alt((peek(host_line), complete_line_ending));
    many_till(property_line, parser)(i)
}

fn host_block(i: &str) -> IResult<&str, Host> {
    let parser = tuple((host_line, properties));
    let (input, (host, p)) = parser(i)?;
    let (props, _) = p;

    let host_struct = Host {
        name: host,
        properties: props,
    };

    Ok((input, host_struct))
}

fn hosts(i: &str) -> IResult<&str, Vec<Host>> {
    many0(host_block)(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_line_newline() {
        let (input, host) =
            host_line("Host dev\n").expect("Could not parse host line ending in '\\n'");
        assert_eq!("", input);
        assert_eq!("dev", host);

        let (input, host) =
            host_line("Host dev-man\r\n").expect("Could not parse host line ending in '\\r\\n'");
        assert_eq!("", input);
        assert_eq!("dev-man", host);
    }

    #[test]
    fn host_line_no_newline() {
        let (input, host) =
            host_line("Host dev").expect("Could not parse host line without terminating newline");
        assert_eq!("", input);
        assert_eq!("dev", host);
    }
}
