use http::{HeaderName, HeaderValue};
use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

/// Represents a HTTP header.
#[derive(Debug, Clone)]
pub struct Header {
    pub field: HeaderName,
    pub value: HeaderValue,
}

impl Header {
    /// Builds a `Header` from two `Vec<u8>`s or two `&[u8]`s.
    ///
    /// Example:
    ///
    /// ```
    /// let header = tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/plain"[..]).unwrap();
    /// ```
    #[allow(clippy::result_unit_err)]
    pub fn from_bytes<B1, B2>(header: B1, value: B2) -> Result<Header, ()>
    where
        B1: AsRef<[u8]>,
        B2: Into<Vec<u8>> + AsRef<[u8]>,
    {
        let header = HeaderName::from_bytes(header.as_ref()).or(Err(()))?;
        let value = HeaderValue::from_bytes(value.as_ref()).or(Err(()))?;

        Ok(Header {
            field: header,
            value,
        })
    }
}

impl FromStr for Header {
    type Err = ();

    fn from_str(input: &str) -> Result<Header, ()> {
        let mut elems = input.splitn(2, ':');

        let field = elems.next().and_then(|f| f.parse().ok()).ok_or(())?;
        let value = elems
            .next()
            .and_then(|v| HeaderValue::from_str(v.trim()).ok())
            .ok_or(())?;

        Ok(Header { field, value })
    }
}

impl Display for Header {
    fn fmt(&self, _formatter: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        // XXX(cosmic): `http` likely intentionally doesn't impl this, so we probably shouldn't
        // either
        todo!();
    }
}

/// HTTP version (usually 1.0 or 1.1).
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HTTPVersion(pub u8, pub u8);

impl Display for HTTPVersion {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(formatter, "{}.{}", self.0, self.1)
    }
}

impl Ord for HTTPVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        let HTTPVersion(my_major, my_minor) = *self;
        let HTTPVersion(other_major, other_minor) = *other;

        if my_major != other_major {
            return my_major.cmp(&other_major);
        }

        my_minor.cmp(&other_minor)
    }
}

impl PartialOrd for HTTPVersion {
    fn partial_cmp(&self, other: &HTTPVersion) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq<(u8, u8)> for HTTPVersion {
    fn eq(&self, &(major, minor): &(u8, u8)) -> bool {
        self.eq(&HTTPVersion(major, minor))
    }
}

impl PartialEq<HTTPVersion> for (u8, u8) {
    fn eq(&self, other: &HTTPVersion) -> bool {
        let &(major, minor) = self;
        HTTPVersion(major, minor).eq(other)
    }
}

impl PartialOrd<(u8, u8)> for HTTPVersion {
    fn partial_cmp(&self, &(major, minor): &(u8, u8)) -> Option<Ordering> {
        self.partial_cmp(&HTTPVersion(major, minor))
    }
}

impl PartialOrd<HTTPVersion> for (u8, u8) {
    fn partial_cmp(&self, other: &HTTPVersion) -> Option<Ordering> {
        let &(major, minor) = self;
        HTTPVersion(major, minor).partial_cmp(other)
    }
}

impl From<(u8, u8)> for HTTPVersion {
    fn from((major, minor): (u8, u8)) -> HTTPVersion {
        HTTPVersion(major, minor)
    }
}

#[cfg(test)]
mod test {
    use super::Header;
    use httpdate::HttpDate;
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_parse_header() {
        let header: Header = "Content-Type: text/html".parse().unwrap();

        assert_eq!(header.field, http::header::CONTENT_TYPE);
        assert_eq!(header.value, "text/html");

        assert!("hello world".parse::<Header>().is_err());
    }

    #[test]
    fn formats_date_correctly() {
        let http_date = HttpDate::from(SystemTime::UNIX_EPOCH + Duration::from_secs(420895020));

        assert_eq!(http_date.to_string(), "Wed, 04 May 1983 11:17:00 GMT")
    }

    #[test]
    fn test_parse_header_with_doublecolon() {
        let header: Header = "Time: 20: 34".parse().unwrap();

        assert_eq!(header.field, "time");
        assert_eq!(header.value.to_str().unwrap(), "20: 34");
    }

    // This tests reslstance to RUSTSEC-2020-0031: "HTTP Request smuggling
    // through malformed Transfer Encoding headers"
    // (https://rustsec.org/advisories/RUSTSEC-2020-0031.html).
    #[test]
    fn test_strict_headers() {
        assert!("Transfer-Encoding : chunked".parse::<Header>().is_err());
        assert!(" Transfer-Encoding: chunked".parse::<Header>().is_err());
        assert!("Transfer Encoding: chunked".parse::<Header>().is_err());
        assert!(" Transfer\tEncoding : chunked".parse::<Header>().is_err());
        assert!("Transfer-Encoding: chunked".parse::<Header>().is_ok());
        assert!("Transfer-Encoding: chunked ".parse::<Header>().is_ok());
        assert!("Transfer-Encoding:   chunked ".parse::<Header>().is_ok());
    }
}
