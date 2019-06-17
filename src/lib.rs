// Many incorrect assumptions were made when creating this initially.
// See the following for a better description on the format:
// https://www.cyberciti.biz/faq/create-ssh-config-file-on-linux-unix/
// https://linux.die.net/man/5/ssh_config

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{line_ending, multispace0, not_line_ending, space0, space1},
    combinator::{map, not, opt, peek},
    multi::many0,
    sequence::tuple,
    IResult,
};

#[derive(PartialEq, Debug)]
pub struct Host<'a> {
    name: &'a str,
    properties: Vec<Property<'a>>,
}

#[derive(PartialEq, Debug)]
pub struct Property<'a> {
    key: &'a str,
    value: &'a str,
}

pub fn parse(data: &str) -> Result<Vec<Host>, ()> {
    hosts(data).map(|(_, hosts)| hosts).map_err(|_| ())
}

fn string(i: &str) -> IResult<&str, &str> {
    take_while1(|c: char| !c.is_whitespace() && c != '#')(i)
}

fn comment(i: &str) -> IResult<&str, &str> {
    let parser = tuple((tag("#"), not_line_ending, opt(line_ending)));
    let (input, (_, _, _)) = parser(i)?;

    Ok((input, ""))
}

fn space_or_comment0(i: &str) -> IResult<&str, &str> {
    let comment_or_whitespace = alt((comment, multispace0));
    let parser = many0(comment_or_whitespace);
    let (input, _) = parser(i)?;

    Ok((input, ""))
}

fn not_comment(i: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c != '\r' && c != '\n' && c != '#')(i)
}

fn host_line(i: &str) -> IResult<&str, &str> {
    let parser = tuple((
        space0,
        tag("Host"),
        space1,
        string,
        space0,
        opt(line_ending),
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
    not(peek(host_line))(i)?;

    let parser = tuple((space0, string, space1, not_comment, opt(line_ending)));

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
    let parser = many0(tuple((space_or_comment0, property_line)));

    let (input, props) = map(parser, |props| props.into_iter().map(|(_, p)| p).collect())(i)?;

    Ok((input, props))
}

fn host_block(i: &str) -> IResult<&str, Host> {
    let parser = tuple((space_or_comment0, host_line, properties));
    let (input, (_, host_name, properties)) = parser(i)?;

    let host = Host {
        name: host_name,
        properties,
    };

    Ok((input, host))
}

fn hosts(i: &str) -> IResult<&str, Vec<Host>> {
    let parser = many0(tuple((space_or_comment0, host_block)));
    let (input, hosts) = map(parser, |hosts| hosts.into_iter().map(|(_, h)| h).collect())(i)?;

    Ok((input, hosts))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string() {
        if string("").is_ok() {
            panic!("Should not be able to parse empty string as valid string");
        }
    }

    #[test]
    fn newline_string() {
        if string("\n").is_ok() {
            panic!("Should not be able to parse newline as valid string");
        }
    }

    #[test]
    fn string_all_whitespace() {
        if string("      ").is_ok() {
            panic!("Should not be able to parse all-spaces as valid string");
        }
    }

    #[test]
    fn string_begins_with_whitespace() {
        if string("   this").is_ok() {
            panic!("Should not be able to parse leading-whitespace string as valid string");
        }
    }

    #[test]
    fn string_ends_in_whitespace() {
        let (input, value) = string("test-str   ").expect("Could not parse string");

        assert_eq!("   ", input);
        assert_eq!("test-str", value);
    }

    #[test]
    fn string_no_whitespace() {
        let good_input = "asd123-456...\\[]";
        let (input, value) = string(good_input).expect("Could not parse valid complicated string");

        assert_eq!("", input);
        assert_eq!(good_input, value);
    }

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
        .expect("Could not parse property collection");

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
        .expect("Could not parse property collection");

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
        .expect("Could not parse single host block pair");

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
        .expect("Could not parse multiple empty hosts");

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

    #[test]
    fn many_hosts() {
        let (input, hosts) = hosts(
            "\n\n\n\n     Host old    \n\
            Asd    123\n\
            Test zz\
            \n\n\n\n\n\n\n\
            Host gregg\n
            HostName hello\n\n\n\n
            Other thing\n\n\n",
        )
        .expect("Could not parse multple hosts with their properties");

        let expected_hosts = vec![
            Host {
                name: "old",
                properties: vec![
                    Property {
                        key: "Asd",
                        value: "123",
                    },
                    Property {
                        key: "Test",
                        value: "zz",
                    },
                ],
            },
            Host {
                name: "gregg",
                properties: vec![
                    Property {
                        key: "HostName",
                        value: "hello",
                    },
                    Property {
                        key: "Other",
                        value: "thing",
                    },
                ],
            },
        ];

        assert_eq!("", input);
        assert_eq!(expected_hosts, hosts);
    }

    #[test]
    fn no_hosts() {
        let empty_input = "       ";
        let (input, hosts) = hosts(empty_input).expect("Could not parse empty string");
        let expected_hosts: Vec<Host> = vec![];

        assert_eq!(empty_input, input);
        assert_eq!(expected_hosts, hosts);
    }

    #[test]
    fn property_as_host_line() {
        if property_line("       \n\nAsd 123\n\n\n").is_ok() {
            panic!("Property is not allowed to be a host line");
        }
    }

    #[test]
    fn proptery_as_host_block() {
        if host_block("       \n\nAsd 123\n\n\n").is_ok() {
            panic!("Property is not allowed to be a host block");
        }
    }

    #[test]
    fn proptery_as_hosts() {
        let bad_input = "       \n\nAsd 123\n\n\n";
        let (input, hosts) =
            hosts(bad_input).expect("Could not parse invalid string for host collection");

        let expected_hosts: Vec<Host> = vec![];

        assert_eq!(bad_input, input);
        assert_eq!(expected_hosts, hosts);
    }

    #[test]
    fn comsume_comment() {
        let (input, result) = comment("#this is a comment").expect("Could not parse comment line");

        assert_eq!("", input);
        assert_eq!("", result);
    }

    #[test]
    fn ignore_comment() {
        let (input, _) = comment("# IGNORE ME\nHost asd\nLocal something")
            .expect("Could not parse comment data");

        assert_eq!("Host asd\nLocal something", input);
    }

    /*#[test]
    fn ignore_comments_and_whitespace() {
        let (input, _) = comment_or_whitespace("\
            \n\
            \n\
            #IGNORE ME\n\
            #      YUUUUUP
            \n\
            \n           \
            #NOPE
            \n\
            Host asd\n\
            Local something").expect("Could not parse comment data");

        assert_eq!("Host asd\nLocal something", input);
    }*/
}
