use nom::{
    bytes::streaming::{tag, take_while1, take_while},
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

    println!("{:#?}", host_block(&data));
}

#[derive(Debug)]
struct Host<'a> {
    name: &'a str,
    properties: Vec<(&'a str, &'a str)>
}

fn whitespace(i: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_whitespace())(i)
}

fn maybe_whitespace(i: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_whitespace())(i)
}

fn line_end(i: &str) -> IResult<&str, &str> {
    tag("\n")(i)
}

fn string(i: &str) -> IResult<&str, &str> {
    take_while1(|c: char| !c.is_whitespace())(i)
}

fn host_line(i: &str) -> IResult<&str, &str> {
    let host = tag("Host");
    let parser = tuple((maybe_whitespace, host, whitespace, string, line_end));

    let (input, (_, _, _, name, _)) = parser(i)?;

    Ok((input, name))
}

fn property_line(i: &str) -> IResult<&str, (&str, &str)> {
    let parser = tuple((maybe_whitespace, string, whitespace, string, line_end));
    let (input, (_, key, _, value, _)) = parser(i)?;

    Ok((input, (key, value)))
}

fn host_block(i: &str) -> IResult<&str, Host> {
    let parser = tuple((host_line, property_line));
    let (input, (host, property)) = parser(i)?;

    let host_struct = Host { name: host, properties: vec![property] };
    Ok((input, host_struct))
}