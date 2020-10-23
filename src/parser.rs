use crate::{checksum, types::Checksum, ve_error::VeError};

#[derive(Debug)]
pub struct Field<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

/// Parse TEXT protocol using nom
fn rawparse(data: &[u8]) -> nom::IResult<&[u8], (Vec<Field>, Checksum)> {
    use nom::bytes::streaming::tag;
    use nom::bytes::streaming::take_until;
    use nom::character::streaming::anychar;
    use nom::character::streaming::char;
    use nom::combinator::map;
    use nom::combinator::not;

    use nom::multi::many1;
    use nom::sequence::pair;
    use nom::sequence::preceded;
    use nom::sequence::separated_pair;
    use nom::IResult;

    // Label which is not "Checksum"
    fn field_label(input: &[u8]) -> IResult<&[u8], &str> {
        map(
            preceded(not(tag("Checksum")), take_until("\t")),
            |s: &[u8]| std::str::from_utf8(s).expect("label"),
        )(input)
    }
    fn line(input: &[u8]) -> IResult<&[u8], Field> {
        // Each field starts with newline, then <field-label> <tab> <field-value>
        let parsed = pair(
            // Newline∏
            tag("\r\n"),
            // Field, tab, value
            separated_pair(field_label, char('\t'), take_until("\r\n")),
        );
        // Map data
        let f = map(parsed, |(_nl, d)| Field {
            key: (d.0),
            value: std::str::from_utf8(d.1).expect("invalid string"),
        })(input);
        f
    }

    fn checksum(input: &[u8]) -> IResult<&[u8], Checksum> {
        // "Checksum" <tab> <checksum byte>
        let parsed = pair(
            tag("\r\n"),
            separated_pair(tag("Checksum"), char('\t'), anychar),
        );
        map(parsed, |(_nl, d)| d.1 as Checksum)(input)
    }

    /// A chunk is a set of fields (at east one) terminated by a checksum.
    /// A chunk may not be a complete frame.
    fn chunk(input: &[u8]) -> IResult<&[u8], (Vec<Field>, Checksum)> {
        pair(many1(line), checksum)(input)
    }

    chunk(data)
}

pub fn parse(data: &[u8]) -> Result<(Vec<Field>, u8, &[u8]), VeError> {
    let (parsed, remainder) = match rawparse(data) {
        Err(nom::Err::Error(e)) => Err(VeError::Parse(format!(
            "Parse error: {:?} - remaining data: {:?}",
            e.1,
            std::str::from_utf8(e.0),
        ))),
        Err(nom::Err::Incomplete(_needed)) => Err(VeError::NeedMoreData),
        Err(e) => Err(VeError::Parse(format!(
            "Unknown error while parsing: {}",
            e
        ))),
        Ok((remainder, parsed)) => Ok((parsed, remainder)),
    }?;

    let (fields, checksum) = parsed;
    let (data, _left) = data.split_at(data.len() - remainder.len());

    match checksum::calculate_for_frame(data) {
        _x if _x == checksum => Ok((fields, checksum, &remainder)),
        bad_checksum => Err(VeError::Parse(format!(
            "Invalid checksum. Expected {}, got {}. The frame is {:?}",
            bad_checksum, checksum, data
        ))),
    }
}

/// Returns the index of the first `\r\n` to detect what could be the start of a frame
fn find_start(data: &[u8]) -> Option<usize> {
    let mut previous: Option<&u8> = None;
    let mut index = 0;
    let mut res: Option<usize> = None;

    for c in data.iter() {
        if previous == Some(&13) && c == &10 {
            res = Some(index - 1);
            break;
        }

        previous = Some(c);
        index += 1;
    }
    res
}

/// Truncate some data to ensure we start at the beginning of a frame.
/// it basically finds the next '\r\n'
fn truncate(data: &[u8]) -> (&[u8], usize) {
    let start = find_start(data);
    println!("Start: {:?}", start);
    match start {
        None => (data, 0),
        Some(i) => (&data[i..], i),
    }
}

/// This function allows finding a valid frame if any is present.
/// If a valid frame (=with a valid checksum) is found, the indexes of start and end in the slice are returned.
pub fn find_frame(data: &[u8]) -> Option<(usize, usize)> {
    // First we truncate the data as needed to make sure we start from a point that could be a valid frame
    let (truncated, start) = truncate(data);

    // From there, we attempt to parse the data
    let parser_result = parse(truncated);

    // If the parser found something, we dig further
    match parser_result {
        Err(_e) => None,
        Ok((_fields, _checksum, remainder)) if remainder.len() == 0 => {
            Some((start, data.len() - start))
        }
        Ok((_fields, _checksum, remainder)) => Some((start, data.len() - remainder.len() - start)),
    }
}

#[cfg(test)]
mod tests_parser {
    use super::*;

    #[test]
    fn test_parse_line() {
        let data = "\r\nfield1\tvalue1\r\nfield2\tvalue2\r\nChecksum\te".as_bytes();
        let (data, _checksum, _remaining) = parse(data).unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0].key, "field1");
        assert_eq!(data[0].value, "value1");
        assert_eq!(data[1].key, "field2");
        assert_eq!(data[1].value, "value2");
    }

    #[test]
    fn test_parse_serial() {
        let frame = "\r\nSER#\tABC123\r\nChecksum\t".as_bytes();
        let frame = checksum::append(frame, 36);

        let (fields, _checksum, _remaining) = parse(&frame).unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].key, "SER#");
        assert_eq!(fields[0].value, "ABC123");
    }

    #[test]
    fn test_parse_hex() {
        let frame = "\r\nPID\t0x1234\r\nChecksum\t".as_bytes();
        let frame = checksum::append(frame, 62);
        let (fields, _checksum, _remaining) = parse(&frame).unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].key, "PID");
        assert_eq!(fields[0].value, "0x1234");
    }

    #[test]
    #[ignore = "Since Victron decided to throw which chars as field names (ie SER#), we need to revise the non-sense unforntunately"]
    fn test_parse_nonsense() {
        let data = "\r\n!!!!\t\tvalue1\r\nChecksum\t42".as_bytes();
        assert!(parse(data).is_err());
    }

    #[test]
    #[ignore = "The current implementation is not ready for this test"]
    fn test_partial_stream() {
        todo!("Fix implementation, this does not pass tests yet");
        let mut data = "\r\nH18\t2415\r\nChecksum\t\u{4}\r\nPID\t0xA381\r\nV\t12282\r\nVS\t29\r\nI\t-2288\r\nP\t-28\r\nCE\t-74900\r\nSOC\t916\r\nTTG\t10350\r\nAlarm\tOFF\r\nRelay\tOFF\r\nAR\t0\r\nBMV\t712 Smart\r\nFW\t0403\r\nChecksum\t~\r\nH1\t-76138\r\nH2\t-76138\r\nH3\t0\r\nH4\t0\r\nH5\t0\r\nH6\t-1876218\r\nH7\t12171\r\nH8\t20418\r\nH9\t1199744\r\nH10\t0\r\nH11\t0\r\nH12\t0\r\nH15\t20\r\nH16\t21033\r\nH17\t2404\r\nH18\t2415\r\nChecksum\t\u{3}\r\nPID\t0xA381\r\n".as_bytes();
        let mut alldata: Vec<Vec<Field>> = vec![];
        while data.len() > 0 {
            let res = parse(&data);
            match res {
                Ok((parsed, _checksum, remainder)) => {
                    alldata.push(parsed);
                    data = remainder;
                }
                Err(VeError::NeedMoreData) => {
                    break;
                }
                Err(e) => {
                    panic!(e);
                }
            };
        }

        // Got three blocks of data
        assert_eq!(alldata.len(), 3);
        // Should have some data remaining
        assert!(data.len() > 0);
        assert_eq!(data.len(), 14);
    }
}

#[cfg(test)]
mod test_frame_finder {
    use super::*;

    #[test]
    fn test_findstart_none() {
        assert_eq!(find_start("f1\tv1\n\rf2".as_bytes()), None);
    }

    #[test]
    fn test_findstart_some_0() {
        assert_eq!(find_start("\r\nf1\tv1\r\nf2".as_bytes()), Some(0));
    }

    #[test]
    fn test_findstart_some() {
        assert_eq!(
            find_start("foo\rbar\nfb\r\nf1\tv1\r\nf2\tv2".as_bytes()),
            Some(10)
        );
    }

    #[test]
    fn test_truncate_nothing() {
        let frame = "\r\nfield1\tvalue1\r\nfield2\tvalue2\r\nChecksum\te".as_bytes();
        assert_eq!(truncate(frame), (frame, 0));
    }

    #[test]
    fn test_truncate_some() {
        let frame =
            "some\rjunk\nfoobar\r\nfield1\tvalue1\r\nfield2\tvalue2\r\nChecksum\te".as_bytes();
        let truncated = truncate(frame);
        assert_eq!(
            truncated,
            (
                "\r\nfield1\tvalue1\r\nfield2\tvalue2\r\nChecksum\te".as_bytes(),
                16
            )
        );
    }

    #[test]
    fn test_find_frame_noframe() {
        let frame = vec![0x0d, 0x0a, 0xd8];
        let result = find_frame(&frame);
        assert_eq!(result, None);
    }

    #[test]
    fn test_find_frame_only_frame() {
        let frame = "\r\nfield1\tvalue1\r\nfield2\tvalue2\r\nChecksum\te".as_bytes();
        assert_eq!(find_frame(&frame), Some((0, 42)));
    }

    #[test]
    fn test_find_frame_junk_then_frame() {
        let frame = "ksum\tf\r\nfield1\tvalue1\r\nfield2\tvalue2\r\nChecksum\te".as_bytes();
        let result = find_frame(&frame);
        assert_eq!(result, Some((6, 42)));
    }

    #[test]
    fn test_find_frame_frame_then_junk() {
        let frame = "\r\nfield1\tvalue1\r\nfield2\tvalue2\r\nChecksum\te\r\nfiel".as_bytes();
        let result = find_frame(&frame);
        assert_eq!(result, Some((0, 42)));
    }

    #[test]
    fn test_find_frame_burried_frame() {
        let frame = "ksum\tf\r\nfield1\tvalue1\r\nfield2\tvalue2\r\nChecksum\te\r\nfiel".as_bytes();
        let result = find_frame(&frame);
        assert_eq!(result, Some((6, 42)));
    }
}
