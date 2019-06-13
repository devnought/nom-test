// Many incorrect assumptions were made when creating this initially.
// See the following for a better description on the format:
// https://www.cyberciti.biz/faq/create-ssh-config-file-on-linux-unix/
// https://linux.die.net/man/5/ssh_config

use nom::{
    bytes::complete::{tag, take_while1},
    character::complete::{line_ending, multispace0, not_line_ending, space0, space1},
    combinator::{map, not, opt, peek},
    multi::many0,
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

#[derive(PartialEq, Debug)]
struct Host<'a> {
    name: &'a str,
    properties: Vec<Property<'a>>,
}

#[derive(PartialEq, Debug)]
struct Property<'a> {
    key: &'a str,
    value: &'a str,
}

fn string(i: &str) -> IResult<&str, &str> {
    take_while1(|c: char| !c.is_whitespace())(i)
}

fn host_line(i: &str) -> IResult<&str, &str> {
    let host = tag("Host");
    let parser = tuple((space0, host, space1, string, space0, opt(line_ending)));

    let (input, (_, _, _, name, _, _)) = parser(i)?;

    Ok((input, name))
}

// TODO: Property should become something like
// Property { key, tokens }
// or
// Property { key, values }
// to handle the case where a key has multple values, like for `LocalForward`
fn property_line(i: &str) -> IResult<&str, Property> {
    not(peek(host_line))(i)?;

    let parser = tuple((space0, string, space1, not_line_ending, opt(line_ending)));

    let (input, (_, key, _, value, _)) = parser(i)?;

    Ok((
        input,
        Property {
            key,
            value: value.trim(),
        },
    ))
}

fn properties(i: &str) -> IResult<&str, Vec<Property>> {
    let parser = many0(tuple((multispace0, property_line, multispace0)));

    let (input, props) = map(parser, |props| {
        props.into_iter().map(|(_, p, _)| p).collect()
    })(i)?;

    Ok((input, props))
}

fn host_block(i: &str) -> IResult<&str, Host> {
    let parser = tuple((multispace0, host_line, properties, multispace0));
    let (input, (_, host_name, properties, _)) = parser(i)?;

    let host = Host {
        name: host_name,
        properties,
    };

    Ok((input, host))
}

fn hosts(i: &str) -> IResult<&str, Vec<Host>> {
    let parser = many0(tuple((multispace0, host_block, multispace0)));
    let (input, hosts) = map(parser, |hosts| {
        hosts.into_iter().map(|(_, h, _)| h).collect()
    })(i)?;

    Ok((input, hosts))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_line_newline_linefeed() {
        let (input, host) =
            host_line("Host dev\n").expect("Could not parse host line ending in '\\n'");
        assert_eq!("", input);
        assert_eq!("dev", host);
    }

    #[test]
    fn host_line_newline_carriagereturn_linefeed() {
        let (input, host) =
            host_line("Host dev-man\r\n").expect("Could not parse host line ending in '\\r\\n'");
        assert_eq!("", input);
        assert_eq!("dev-man", host);
    }

    #[test]
    fn host_line_no_newline() {
        let (input, host) =
            host_line("Host dev").expect("Could not parse host line with no newline");
        assert_eq!("", input);
        assert_eq!("dev", host);
    }

    #[test]
    fn property_line_newline() {
        let (input, property) =
            property_line("      LocalForward      9906 127.0.0.1:3306        \n")
                .expect("Could not parse property line with line ending in '\\n'");

        let expected_property = Property {
            key: "LocalForward",
            value: "9906 127.0.0.1:3306",
        };

        assert_eq!("", input);
        assert_eq!(expected_property, property);
    }

    #[test]
    fn multiple_properties() {
        let (input, properties) = properties(
            "   \n\n\n HostName database.example.com\n    IdentityFile ~/.ssh/coolio.example.key\n\n\n\n\n\n\nAsd 123",
        )
        .expect("Coult not parse property collection");

        let expected_properties = vec![
            Property {
                key: "HostName",
                value: "database.example.com",
            },
            Property {
                key: "IdentityFile",
                value: "~/.ssh/coolio.example.key",
            },
            Property {
                key: "Asd",
                value: "123",
            },
        ];

        assert_eq!("", input);
        assert_eq!(expected_properties, properties);
    }

    #[test]
    fn multiple_properties_hostline_end() {
        let (input, properties) = properties(
            "   \n\n\n HostName database.example.com\n     \
             IdentityFile ~/.ssh/coolio.example.key\
             \n\n\n\n\n\n\n\
             Asd 123\n      \
             Host devv\n\n",
        )
        .expect("Coult not parse property collection");

        let expected_properties = vec![
            Property {
                key: "HostName",
                value: "database.example.com",
            },
            Property {
                key: "IdentityFile",
                value: "~/.ssh/coolio.example.key",
            },
            Property {
                key: "Asd",
                value: "123",
            },
        ];

        assert_eq!("Host devv\n\n", input);
        assert_eq!(expected_properties, properties);
    }

    #[test]
    fn single_host_block() {
        let (input, host) = host_block(
            "  \n\n   \n\n\
             \n       Host dev\n   \
             Asd      123     \n\n",
        )
        .expect("Could not parse single host block");

        let expected_host = Host {
            name: "dev",
            properties: vec![Property {
                key: "Asd",
                value: "123",
            }],
        };

        assert_eq!("", input);
        assert_eq!(expected_host, host);
    }

    #[test]
    fn single_host_block_no_properties() {
        let (input, host) = host_block(
            "  \n\n   \n\n\
             \n       Host dev\n   \
             \n\n",
        )
        .expect("Could not parse single host block");

        let expected_host = Host {
            name: "dev",
            properties: vec![],
        };

        assert_eq!("", input);
        assert_eq!(expected_host, host);
    }

    #[test]
    fn two_host_blocks_no_properties() {
        let (input, host) = host_block(
            "  \n\n   \n\n\
             \n       Host dev\n   \
             \n\n\
             Host zzz",
        )
        .expect("Could not parse single host block");

        let expected_host = Host {
            name: "dev",
            properties: vec![],
        };

        assert_eq!("Host zzz", input);
        assert_eq!(expected_host, host);
    }

    #[test]
    fn many_hosts_no_properties() {
        let (input, hosts) = hosts(
            "  \n\n   \n\n\
             \n       Host dev\n   \
             \n\n\
             Host zzz",
        )
        .expect("Could not parse single host block");

        let expected_hosts = vec![
            Host {
                name: "dev",
                properties: vec![],
            },
            Host {
                name: "zzz",
                properties: vec![],
            },
        ];

        assert_eq!("", input);
        assert_eq!(expected_hosts, hosts);
    }
}
