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
    /// let header = tiny_http::Header::from_bytes(b"Content-Type", b"text/plain").unwrap();
    /// ```
    #[allow(clippy::result_unit_err)]
    pub fn from_bytes<B1, B2>(header: B1, value: B2) -> Result<Header, ()>
    where
        B1: AsRef<[u8]>,
        B2: AsRef<[u8]>,
    {
        let header = HeaderName::from_bytes(header.as_ref()).map_err(|_| ())?;
        let value = HeaderValue::from_bytes(value.as_ref()).map_err(|_| ())?;

        Ok(Header {
            field: header,
            value,
        })
    }
}

impl FromStr for Header {
    type Err = ();

    fn from_str(input: &str) -> Result<Header, ()> {
        let (field, value) = input.split_once(':').ok_or(())?;

        let field = field.parse().map_err(|_| ())?;
        let value = HeaderValue::from_str(value.trim()).map_err(|_| ())?;

        Ok(Header { field, value })
    }
}

impl Display for Header {
    fn fmt(&self, _formatter: &mut Formatter<'_>) -> fmt::Result {
        // XXX(cosmic): `http` likely intentionally doesn't impl this, so we probably shouldn't
        // either
        todo!();
    }
}

/// HTTP version (usually 1.0 or 1.1).
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HttpVersion(pub u8, pub u8);

impl Display for HttpVersion {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}", self.0, self.1)
    }
}

impl PartialEq<(u8, u8)> for HttpVersion {
    fn eq(&self, &(major, minor): &(u8, u8)) -> bool {
        self.eq(&HttpVersion(major, minor))
    }
}

impl PartialEq<HttpVersion> for (u8, u8) {
    fn eq(&self, other: &HttpVersion) -> bool {
        let us: HttpVersion = (*self).into();
        us.eq(other)
    }
}

impl PartialOrd<(u8, u8)> for HttpVersion {
    fn partial_cmp(&self, &(major, minor): &(u8, u8)) -> Option<Ordering> {
        self.partial_cmp(&HttpVersion(major, minor))
    }
}

impl PartialOrd<HttpVersion> for (u8, u8) {
    fn partial_cmp(&self, other: &HttpVersion) -> Option<Ordering> {
        let us: HttpVersion = (*self).into();
        us.partial_cmp(other)
    }
}

impl From<(u8, u8)> for HttpVersion {
    fn from((major, minor): (u8, u8)) -> HttpVersion {
        HttpVersion(major, minor)
    }
}

// TODO: fix typos picked up by `typos`
// TODO: init logging in tests
// TODO: group integration tests into one binary
// TODO: I broke the benches. Should probably fix those...
// TODO: disallow the log macros that have overrides, so that we can pick it up early?

#[cfg(test)]
mod test {
    use super::Header;
    use http::header;
    use httpdate::HttpDate;
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_parse_header() {
        let header: Header = "Content-Type: text/html".parse().unwrap();

        assert_eq!(header.field, header::CONTENT_TYPE);
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

    // This tests resistance to RUSTSEC-2020-0031: "HTTP Request smuggling
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
