// Many incorrect assumptions were made when creating this initially.
// See the following for a better description on the format:
// https://www.cyberciti.biz/faq/create-ssh-config-file-on-linux-unix/
// https://linux.die.net/man/5/ssh_config

use nom::{
    branch::alt,
    bytes::streaming::{tag, take_while, take_while1},
    combinator::peek,
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

fn whitespace_def(c: char) -> bool {
    c != '\r' && c != '\n' && c.is_whitespace()
}

fn whitespace(i: &str) -> IResult<&str, &str> {
    take_while1(whitespace_def)(i)
}

fn maybe_whitespace(i: &str) -> IResult<&str, &str> {
    take_while(whitespace_def)(i)
}

fn line_end(i: &str) -> IResult<&str, &str> {
    let crlf = tag("\r\n");
    let lf = tag("\n");
    let ending = alt((crlf, lf));
    let parser = tuple((maybe_whitespace, ending));

    let (input, (_, end)) = parser(i)?;

    Ok((input, end))
}

fn string(i: &str) -> IResult<&str, &str> {
    take_while1(|c: char| !c.is_whitespace())(i)
}

fn string_not_eol(i: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c != '\r' && c != '\n')(i)
}

fn host_line(i: &str) -> IResult<&str, &str> {
    let host = tag("Host");
    let parser = tuple((maybe_whitespace, host, whitespace, string, line_end));

    let (input, (_, _, _, name, _)) = parser(i)?;

    Ok((input, name))
}

// TODO: Property should become something like
// Property { key, tokens }
// or
// Property { key, values }
fn property_line(i: &str) -> IResult<&str, Property> {
    let parser = tuple((maybe_whitespace, string, whitespace, string_not_eol, line_end));
    let (input, (_, key, _, value, _)) = parser(i)?;

    Ok((input, Property { key, value }))
}

fn properties(i: &str) -> IResult<&str, (Vec<Property>, &str)> {
    let parser = alt((peek(host_line), line_end));
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
