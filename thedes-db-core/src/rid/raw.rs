use thiserror::Error;

// [a-z0-9]{3}/[a-zA-Z0-9]:[a-z0-9_#]{20}
//
// <namespace>/<tag>:<data>
//
// <data> has 104 bits of maximum entropy

pub type Bits = u128;

pub const ENCODED_SIZE: usize = 3 + 1 + 1 + 1 + 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum Error {
    #[error("incomplete namespace")]
    IncompleteNamespace,
    #[error("bad namespace character")]
    BadNamespaceChar,
    #[error("missing namespace separator")]
    MissingNamespaceSep,
    #[error("missing entity tag")]
    MissingTag,
    #[error("bad tag character")]
    BadTag,
    #[error("missing data separator")]
    MissingDataSep,
    #[error("bad data character")]
    BadDataChar,
    #[error("data is too large")]
    DataTooLarge,
}

pub fn encode(bytes: &[u8]) -> Result<Bits, Error> {
    let mut bytes = bytes.iter().copied();

    let mut encoded: Bits = 0;

    for _ in 0 .. 3 {
        encoded *= 26 + 10;
        let Some(byte) = bytes.next() else { Err(Error::IncompleteNamespace)? };
        if byte.is_ascii_lowercase() {
            encoded += Bits::from(byte - b'a' + 10);
        } else if byte.is_ascii_digit() {
            encoded += Bits::from(byte - b'0');
        } else if byte == b'/' {
            Err(Error::IncompleteNamespace)?
        } else {
            Err(Error::BadNamespaceChar)?
        };
    }

    let Some(b'/') = bytes.next() else { Err(Error::MissingNamespaceSep)? };

    let Some(tag) = bytes.next() else { Err(Error::MissingTag)? };

    encoded *= 26 * 2 + 10;

    if tag.is_ascii_lowercase() {
        encoded += Bits::from(tag - b'a' + 10 + 26);
    } else if tag.is_ascii_uppercase() {
        encoded += Bits::from(tag - b'A');
    } else if tag.is_ascii_digit() {
        encoded += Bits::from(tag - b'0' + 26);
    } else if tag == b':' {
        Err(Error::MissingTag)?
    } else {
        Err(Error::BadTag)?
    };

    let Some(b':') = bytes.next() else { Err(Error::MissingDataSep)? };

    for _ in 0 .. 20 {
        encoded *= 26 + 10 + 1 + 1;

        let byte = bytes.next().unwrap_or(b'%');
        if byte == b'%' {
            encoded += 0;
        } else if byte.is_ascii_digit() {
            encoded += Bits::from(byte - b'0' + 1);
        } else if byte == b'_' {
            encoded += 1 + 10
        } else if byte.is_ascii_lowercase() {
            encoded += Bits::from(byte - b'a' + 1 + 10 + 1);
        } else {
            Err(Error::BadDataChar)?
        }
    }

    if bytes.next().is_some() {
        Err(Error::DataTooLarge)?
    }

    Ok(encoded)
}

pub fn decode(mut raw: Bits, buf: &mut [u8; ENCODED_SIZE]) -> &[u8] {
    let mut i = buf.len();
    let mut tail = i;
    for _ in 0 .. 20 {
        i -= 1;
        let code = (raw % (26 + 10 + 1 + 1)) as u8;
        buf[i] = if code == 0 {
            b'%'
        } else if code >= 1 && code < 1 + 10 {
            code - 1 + b'0'
        } else if code == 11 {
            b'_'
        } else {
            code - 12 + b'a'
        };
        raw /= 26 + 10 + 1 + 1;
        if tail - 1 == i && buf[i] == b'%' {
            tail = i;
        }
    }
    i -= 1;
    buf[i] = b':';
    i -= 1;
    let code = (raw % (26 * 2 + 10)) as u8;
    buf[i] = if code < 26 {
        code + b'A'
    } else if code >= 26 && code < 26 + 10 {
        code - 26 + b'0'
    } else {
        code - 26 - 10 + b'a'
    };
    raw /= 26 * 2 + 10;
    i -= 1;
    buf[i] = b'/';
    for _ in 0 .. 3 {
        i -= 1;
        let code = (raw % (26 + 10)) as u8;
        buf[i] = if code < 10 { code + b'0' } else { code - 10 + b'a' };
        raw /= 26 + 10;
    }
    &buf[.. tail]
}

#[cfg(test)]
mod test {

    fn prop_encode_decode(original: &[u8]) {
        let encoded = super::encode(original).unwrap();
        let mut buf = [0; super::ENCODED_SIZE];
        let decoded = super::decode(encoded, &mut buf);
        assert_eq!(original, decoded);
    }

    #[test]
    fn encode_decode_players() {
        prop_encode_decode(b"th0/t:players");
    }

    #[test]
    fn encode_decode_random() {
        prop_encode_decode(b"th0/o:3yrkihv%u%x_bzt5akm7");
    }

    #[test]
    fn encode_decode_max() {
        prop_encode_decode(b"zzz/z:zzzzzzzzzzzzzzzzzzzz");
    }

    #[test]
    fn encode_decode_one_less_than_max() {
        prop_encode_decode(b"zzz/z:zzzzzzzzzzzzzzzzzzzy");
    }

    #[test]
    fn encode_decode_min() {
        prop_encode_decode(b"000/A:00000000000000000000");
    }

    #[test]
    fn encode_decode_one_more_than_min() {
        prop_encode_decode(b"000/A:00000000000000000001");
    }

    #[test]
    fn encode_decode_tag_allows_number() {
        prop_encode_decode(b"th0/3:abcdef");
    }

    #[test]
    fn encode_decode_tag_allows_0() {
        prop_encode_decode(b"th0/0:abcdef");
    }

    #[test]
    fn encode_decode_tag_allows_9() {
        prop_encode_decode(b"th0/9:abcdef");
    }

    #[test]
    fn encode_decode_tag_allows_upper_z() {
        prop_encode_decode(b"th0/Z:abcdef");
    }

    #[test]
    fn encode_decode_tag_allows_lower_a() {
        prop_encode_decode(b"th0/a:abcdef");
    }

    #[test]
    fn encode_decode_namespace_allows_lower_a() {
        prop_encode_decode(b"ah0/a:abcdef");
    }

    #[test]
    fn encode_decode_namespace_allows_lower_z() {
        prop_encode_decode(b"zh0/a:abcdef");
    }

    #[test]
    fn encode_decode_namespace_allows_9() {
        prop_encode_decode(b"th9/a:abcdef");
    }

    #[test]
    fn encode_decode_data_allows_0() {
        prop_encode_decode(b"th0/a:0");
    }

    #[test]
    fn encode_decode_data_allows_9() {
        prop_encode_decode(b"th0/a:9");
    }

    #[test]
    fn encode_decode_data_allows_empty() {
        prop_encode_decode(b"th0/a:");
    }

    #[test]
    fn does_not_allow_empty_namespace() {
        let result = super::encode(b"/x:abc123");
        assert_eq!(result, Err(super::Error::IncompleteNamespace));
    }

    #[test]
    fn does_not_allow_small_namespace() {
        let result = super::encode(b"ab/x:abc123");
        assert_eq!(result, Err(super::Error::IncompleteNamespace));
    }

    #[test]
    fn does_not_allow_big_namespace() {
        let result = super::encode(b"abcd/x:abc123");
        assert_eq!(result, Err(super::Error::MissingNamespaceSep));
    }

    #[test]
    fn does_not_allow_invalid_namespace_char() {
        let result = super::encode(b"thX/x:abc123");
        assert_eq!(result, Err(super::Error::BadNamespaceChar));
    }

    #[test]
    fn does_not_allow_big_tag() {
        let result = super::encode(b"th0/xy:abc123");
        assert_eq!(result, Err(super::Error::MissingDataSep));
    }

    #[test]
    fn does_not_allow_invalid_tag_char() {
        let result = super::encode(b"th0/@:abc123");
        assert_eq!(result, Err(super::Error::BadTag));
    }

    #[test]
    fn does_not_allow_data_too_large() {
        let result = super::encode(b"th0/x:012345678901234567890");
        assert_eq!(result, Err(super::Error::DataTooLarge));
    }
}
