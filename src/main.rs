use nom::{
    bytes::streaming::{tag, take_while1},
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

    println!("{:#?}", host_line(&data));
}

#[derive(Debug)]
struct Host<'a> {
    name: &'a str,
}

fn host_line(i: &str) -> IResult<&str, Host> {
    let host = tag("Host");
    let space = take_while1(|c: char| c.is_whitespace());
    let name_str = take_while1(|c: char| !c.is_whitespace());
    let line_end = tag("\n");

    let parser = tuple((host, space, name_str, line_end));

    let (input, (_, _, name, _)) = parser(i)?;

    Ok((input, Host { name }))
}
