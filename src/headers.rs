/// HTTP content types

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ContentType {
    TextHtml,
    TextPlain,
    ApplicationJson,
    ApplicationCbor,
    ApplicationOctetStream,
}

impl<'a> From<&'a [u8]> for ContentType {
    fn from(from: &'a [u8]) -> ContentType {
        match from {
            b"application/json" => ContentType::ApplicationJson,
            b"application/cbor" => ContentType::ApplicationCbor,
            b"text/html" => ContentType::TextHtml,
            b"text/plain" => ContentType::TextPlain,
            _ => ContentType::ApplicationOctetStream,
        }
    }
}

impl ContentType {
    pub fn as_str(&self) -> &str {
        match self {
            ContentType::TextHtml => "text/html",
            ContentType::TextPlain => "text/plain",
            ContentType::ApplicationJson => "application/json",
            ContentType::ApplicationCbor => "application/cbor",
            ContentType::ApplicationOctetStream => "application/octet-stream",
        }
    }
}

/// Transfer encoding
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TransferEncoding {
    Chunked,
    Compress,
    Deflate,
    Gzip,
}

impl<'a> TryFrom<&'a [u8]> for TransferEncoding {
    type Error = ();

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(match value {
            b"chunked" => TransferEncoding::Chunked,
            b"compress" => TransferEncoding::Compress,
            b"deflate" => TransferEncoding::Deflate,
            b"gzip" => TransferEncoding::Gzip,
            _ => return Err(()),
        })
    }
}

impl TransferEncoding {
    pub fn as_str(&self) -> &str {
        match self {
            TransferEncoding::Deflate => "deflate",
            TransferEncoding::Chunked => "chunked",
            TransferEncoding::Compress => "compress",
            TransferEncoding::Gzip => "gzip",
        }
    }
}

/// Keep-alive header
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct KeepAlive {
    timeout: Option<u8>,
    max: Option<u8>,
}

impl<'a> TryFrom<&'a [u8]> for KeepAlive {
    type Error = core::str::Utf8Error;

    fn try_from(from: &'a [u8]) -> Result<Self, Self::Error> {
        let mut keep_alive = KeepAlive {
            timeout: None,
            max: None,
        };
        for part in core::str::from_utf8(from)?.split(',') {
            let mut splitted = part.split('=').map(|s| s.trim());
            if let (Some(key), Some(value)) = (splitted.next(), splitted.next()) {
                match key {
                    _ if key.eq_ignore_ascii_case("timeout") => keep_alive.timeout = value.parse().ok(),
                    _ if key.eq_ignore_ascii_case("max") => keep_alive.max = value.parse().ok(),
                    _ => (),
                }
            }
        }
        Ok(keep_alive)
    }
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg(feature="date")]
pub struct HeaderDate {
    pub date: Option<[u8;14]>  // b"YYYYMMDDhhmmss" UTC
}

#[cfg(feature="date")]
impl<'a> TryFrom<&'a [u8]> for HeaderDate {
    type Error = ();

    fn try_from(from: &'a [u8]) -> Result<Self, Self::Error> {
        // Date: <day-name>, <day> <month> <year> <hour>:<minute>:<second> GMT
        let mut candidate: [u8;14] = [b' '; 14];
        const MONTHS: [&str; 12] = ["Ja", "F", "Mar", "Ap", "May", "Jun", "Jul", "Au", "S", "O", "N", "D"];
        let val = core::str::from_utf8(from);
        let val_str: &str;
        if let Ok(val) = val {
            val_str = val;
        } else {
            return Err(());
        }
        let mut byspace = val_str.split(' ');
        byspace.next(); // skip day-name
        let day = byspace.next();
        if let Some(day) = day {
            if day.len() == 2 {
                let day = day.as_bytes();
                candidate[6] = day[0];
                candidate[7] = day[1];
            }
        }
        let month = byspace.next();
        if let Some(month) = month {
            if month.len() == 3 {
                'mo: for i in 0..=2 {
                    let prefix = &month[0..=i];
                    for (mz, m) in MONTHS.iter().enumerate() {
                        if *m == prefix {
                            let monthnum = mz + 1;
                            candidate[4] = (monthnum as u8 / 10) + b'0';
                            candidate[5] = (monthnum as u8 % 10) + b'0';
                            break 'mo;
                        }
                    }
                }
            }
        }
        let year = byspace.next();
        if let Some(year) = year {
            if year.len() == 4 {
                let year = year.as_bytes();
                candidate[0] = year[0];
                candidate[1] = year[1];
                candidate[2] = year[2];
                candidate[3] = year[3];
            }
        }
        let hms = byspace.next();
        if let Some(hms) = hms {
            if hms.len() == 8 {
                let hms = hms.as_bytes();
                candidate[8] = hms[0];
                candidate[9] = hms[1];
                candidate[10] = hms[3];
                candidate[11] = hms[4];
                candidate[12] = hms[6];
                candidate[13] = hms[7];
            }
        }
        Ok(Self {
            date: Some(candidate),
        })
    }
}
