// Many incorrect assumptions were made when creating this initially.
// See the following for a better description on the format:
// https://www.cyberciti.biz/faq/create-ssh-config-file-on-linux-unix/
// https://linux.die.net/man/5/ssh_config

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{line_ending, multispace0, multispace1, not_line_ending, space0, space1},
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

pub fn parse(_data: &str) -> Result<Vec<Host>, ()> {
    Ok(vec![])
}

fn string(i: &str) -> IResult<&str, &str> {
    take_while1(|c: char| !c.is_whitespace() && c != '#')(i)
}

fn comment(i: &str) -> IResult<&str, &str> {
    let parser = tuple((tag("#"), not_line_ending, opt(line_ending)));
    let (input, (_, _, _)) = parser(i)?;

    Ok((input, ""))
}

fn only_string(i: &str) -> IResult<&str, &str> {
    let whitespace_or_comment = alt((comment, multispace0));
    let parser = tuple((whitespace_or_comment, string));

    let (input, (_, data)) = parser(i)?;

    Ok((input, data))
}

fn strings(i: &str) -> IResult<&str, Vec<&str>> {
    many0(only_string)(i)
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
    fn valid_string() {
        let (input, host) = string("Host").expect("Could not parse 'Host'");

        assert_eq!("", input);
        assert_eq!("Host", host);
    }

    #[test]
    fn valid_string_space() {
        let (input, host) = string("Host dev").expect("Could not parse 'Host dev'");

        assert_eq!(" dev", input);
        assert_eq!("Host", host);
    }

    #[test]
    fn valid_strings_eat_comment() {
        let (input, host) = only_string("Host#dev\nhello").expect("Could not parse 'Host#dev");
        let (input2, hello) = only_string(input).expect("Could not parse result");

        assert_eq!("#dev\nhello", input);
        assert_eq!("Host", host);

        assert_eq!("", input2);
        assert_eq!("hello", hello);
    }

    #[test]
    fn many_strings() {
        let (input, strings) = strings("Host#dev\nhello\n\n\ntest\n\n")
            .expect("Could not parse multiple valid strings with comment");

        let expected_strings = vec!["Host", "hello", "test"];

        assert_eq!("", input);
        assert_eq!(expected_strings, strings);
    }
}
