// Many incorrect assumptions were made when creating this initially.
// See the following for a better description on the format:
// https://www.cyberciti.biz/faq/create-ssh-config-file-on-linux-unix/
// https://linux.die.net/man/5/ssh_config

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{line_ending, multispace0, multispace1, not_line_ending, space0, space1},
    combinator::{map, not, opt, peek},
    multi::{many0, many1},
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
    let parser = tuple((tag("#"), not_line_ending));
    let (input, (_, _)) = parser(i)?;

    Ok((input, ""))
}

fn space_or_equals(i: &str) -> IResult<&str, &str> {
    let parser = alt((tag("="), space1));
    let (input, _) = parser(i)?;

    Ok((input, ""))
}

fn space_or_comment(i: &str) -> IResult<&str, &str> {
    let parser = alt((comment, multispace1));
    let (input, _) = parser(i)?;

    Ok((input, ""))
}

fn spaces_or_comments(i: &str) -> IResult<&str, &str> {
    let parser = many0(space_or_comment);
    let (input, _) = parser(i)?;

    Ok((input, ""))
}

fn not_comment(i: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c != '\r' && c != '\n' && c != '#')(i)
}

fn host_line(i: &str) -> IResult<&str, &str> {
    let parser = tuple((tag("Host"), space_or_equals, string));
    let (input, (_, _, name)) = parser(i)?;

    Ok((input, name))
}

fn property_line(i: &str) -> IResult<&str, Property> {
    not(peek(host_line))(i)?;

    let parser = tuple((string, space_or_equals, not_comment));
    let (input, (key, _, value)) = parser(i)?;

    Ok((
        input,
        Property {
            key,
            value: value.trim(),
        },
    ))
}

fn properties(i: &str) -> IResult<&str, Vec<Property>> {
    let parser = many0(tuple((spaces_or_comments, property_line)));
    let (input, lines) = parser(i)?;
    let mapped_lines = lines.into_iter().map(|(_, x)| x).collect::<Vec<_>>();

    Ok((input, mapped_lines))
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Write tests for equal separators

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
    fn space_or_comment_spaces() {
        let (input, _) = space_or_comment("   ").expect("Could not parse whitespace");
        assert_eq!("", input);
    }

    #[test]
    fn space_or_comment_comment() {
        let (input, _) = space_or_comment("#this is a comment").expect("Could not parse comment");
        assert_eq!("", input);
    }

    #[test]
    fn space_or_comment_spaces_and_comment() {
        let (input, _) =
            space_or_comment("      #comment").expect("Could not parse space and comment");
        assert_eq!("#comment", input);

        let (input, _) = space_or_comment(input).expect("Could not parse remaining comment");
        assert_eq!("", input);
    }

    #[test]
    fn spaces_and_comments_both() {
        let (input, _) = spaces_or_comments("     #comment\n\n\n#comment      \n\n")
            .expect("Could not parse spaces and comment");
        assert_eq!("", input);
    }

    #[test]
    fn many_properties() {
        let (input, properties) = properties("   \n\n\n      asd 123 345\n\n\nDave yes\n")
            .expect("Could not parse properties");

        let expected_properties = vec![
            Property {
                key: "asd",
                value: "123 345",
            },
            Property {
                key: "Dave",
                value: "yes",
            },
        ];

        assert_eq!("\n", input);
        assert_eq!(expected_properties, properties);
    }

    #[test]
    fn many_properties_comments() {
        let data = r"
            HostName butt   #no butts   
            Asd 123
            #moar comment


            Blah whatever";

        let (input, properties) =
            properties(data).expect("Could not parse a mix of properties and comments");

        let expected_properties = vec![
            Property {
                key: "HostName",
                value: "butt",
            },
            Property {
                key: "Asd",
                value: "123",
            },
            Property {
                key: "Blah",
                value: "whatever",
            },
        ];

        assert_eq!("", input);
        assert_eq!(expected_properties, properties);
    }
}
