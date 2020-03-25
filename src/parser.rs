use crate::VEError;

#[derive(Debug)]
pub struct Field<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

#[derive(Debug)]
pub struct Checksum {
    value: u8,
}

/// Parse binary protocol using nom
fn rawparse(data: &[u8]) -> nom::IResult<&[u8], (Vec<Field>, Checksum)> {
    use nom::bytes::complete::tag;
    use nom::character::complete::alphanumeric1;
    use nom::character::complete::anychar;
    use nom::character::complete::char;
    use nom::combinator::map;
    use nom::combinator::not;
    use nom::multi::many1;
    use nom::sequence::pair;
    use nom::sequence::preceded;
    use nom::sequence::separated_pair;
    use nom::IResult;

    fn field_label_or_value(input: &[u8]) -> IResult<&[u8], &str> {
        map(
            preceded(not(tag("Checksum")), alphanumeric1),
            |s: &[u8]| std::str::from_utf8(s).unwrap(),
        )(input)
    }
    fn line(input: &[u8]) -> IResult<&[u8], Field> {
        // Starts with newline
        let (input, _) = tag("\r\n")(input)?;
        // Then <field-label> <tab> <field-value>
        let parsed = separated_pair(field_label_or_value, char('\t'), field_label_or_value);
        // Map data
        map(parsed, |d| Field {
            key: d.0,
            value: d.1,
        })(input)
    }
    fn checksum(input: &[u8]) -> IResult<&[u8], Checksum> {
        // Starts with newline
        let (input, _) = tag("\r\n")(input)?;
        // "Checksum" <tab> <checksum byte>
        let parsed = separated_pair(tag("Checksum"), char('\t'), anychar);
        map(parsed, |d| Checksum { value: d.1 as u8 })(input)
    }
    fn chunk(input: &[u8]) -> IResult<&[u8], (Vec<Field>, Checksum)> {
        pair(many1(line), checksum)(input)
    }
    chunk(data)
}

pub fn parse(data: &[u8]) -> Result<Vec<Field>, VEError> {
    let data = match rawparse(data) {
        Err(nom::Err::Error(e)) => Err(VEError::Parse(format!(
            "Parse error: {:?} - remaining data: {:?}",
            e.1,
            std::str::from_utf8(e.0),
        ))),
        Err(e) => Err(VEError::Parse(format!("Unknown error while parsing: {}", e))),
        Ok((_remainder, data)) => Ok(data),
    }?;
    let (fields, checksum) = data;
    println!("Checksum: {:?}\nFields: {:#?}", checksum, fields);
    Ok(fields)
}

#[test]
fn test_parse_line() {
    let data = "\r\nfield1\tvalue1\r\nfield2\tvalue2\r\nChecksum\t4".as_bytes();
    println!("{:?}", data);
    let data = parse(data).unwrap();
    assert_eq!(data.len(), 2);
    assert_eq!(data[0].key, "field1");
    assert_eq!(data[0].value, "value1");
    assert_eq!(data[1].key, "field2");
    assert_eq!(data[1].value, "value2");
}

#[test]
fn test_parse_nonsense() {
    let data = "\r\n!!!!\t\tvalue1\r\nChecksum\t42".as_bytes();
    assert!(parse(data).is_err());
}
