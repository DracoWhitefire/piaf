/// Manufacture date or model year, decoded from EDID base block bytes 16–17.
///
/// | Byte 16 | Meaning                                              |
/// |---------|------------------------------------------------------|
/// | `0x00`  | Week unspecified; byte 17 is the manufacture year.  |
/// | `0x01`–`0x36` | Week of manufacture (1–54).               |
/// | `0xFF`  | Byte 17 is a model year, not a manufacture year.    |
///
/// Year is encoded as `byte_17 + 1990`.
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManufactureDate {
    /// The display was manufactured in the given year.
    /// `week` is `None` if byte 16 was `0x00` (week unspecified).
    Manufactured {
        /// Week of manufacture (1–54), if specified.
        week: Option<u8>,
        /// Year of manufacture.
        year: u16,
    },
    /// The year identifies a model year rather than a manufacture date.
    ModelYear(u16),
}

impl ManufactureDate {
    /// Decodes bytes 16 and 17 of the EDID base block.
    pub(crate) fn from_edid_bytes(week_byte: u8, year_byte: u8) -> Self {
        let year = year_byte as u16 + 1990;
        match week_byte {
            0xFF => Self::ModelYear(year),
            0x00 => Self::Manufactured { week: None, year },
            w => Self::Manufactured {
                week: Some(w),
                year,
            },
        }
    }
}

/// A three-character PNP manufacturer identifier, decoded from EDID base block bytes `0x08`–`0x09`.
///
/// Each character is an ASCII uppercase letter (A–Z). Valid IDs are registered with the IANA
/// PNP registry. Well-known examples: `GSM` (LG), `SAM` (Samsung), `DEL` (Dell).
///
/// # Invariant
///
/// All three bytes must be ASCII uppercase letters (`b'A'`–`b'Z'`, i.e. `0x41`–`0x5A`).
/// The library only constructs this type after validating that constraint. If you construct
/// one manually via the public field, you are responsible for maintaining the invariant;
/// methods on this type will panic in debug builds if it is violated.
///
/// Use [`ManufacturerId::from_ascii`] for a checked construction path.
///
/// Available in all build configurations including bare `no_std`. The `Display` impl renders
/// the three-character string directly, so `format!("{}", id)` and `id.to_string()` both work
/// wherever a `Display` bound is satisfied.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManufacturerId(pub [u8; 3]);

impl ManufacturerId {
    /// Constructs a `ManufacturerId` from three raw bytes, returning `None` if any byte is
    /// not an ASCII uppercase letter (`A`–`Z`).
    pub fn from_ascii(bytes: [u8; 3]) -> Option<Self> {
        if bytes.iter().all(|&b| b.is_ascii_uppercase()) {
            Some(Self(bytes))
        } else {
            None
        }
    }

    /// Returns the ID as a `&str` slice.
    ///
    /// Panics in debug builds if the stored bytes are not ASCII uppercase letters, which
    /// would indicate the type invariant was violated at construction time.
    pub fn as_str(&self) -> &str {
        debug_assert!(
            self.0.iter().all(|&b| b.is_ascii_uppercase()),
            "ManufacturerId invariant violated: bytes must be ASCII uppercase A-Z, got {:?}",
            self.0
        );
        // Safety: ASCII uppercase bytes are always valid UTF-8.
        core::str::from_utf8(&self.0).expect("ManufacturerId bytes must be ASCII uppercase A-Z")
    }
}

impl core::fmt::Display for ManufacturerId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A monitor descriptor string, decoded from one of the 18-byte descriptor slots in the
/// EDID base block (`0xFC` monitor name, `0xFF` serial number, `0xFE` unspecified text).
///
/// The text payload occupies bytes 5–17 of the descriptor (13 bytes), terminated by `0x0A`
/// and padded with spaces. The `as_str()` method strips both the terminator and trailing
/// spaces, returning a clean `&str`.
///
/// Available in all build configurations including bare `no_std`. `Deref<Target = str>`
/// is implemented so `Option<MonitorString>::as_deref()` returns `Option<&str>` directly.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonitorString(pub [u8; 13]);

impl MonitorString {
    /// Returns the string content with the `0x0A` terminator and trailing spaces stripped.
    ///
    /// Returns an empty `&str` if the payload is all padding or not valid UTF-8.
    pub fn as_str(&self) -> &str {
        let bytes = &self.0;
        let end = bytes.iter().position(|&b| b == 0x0A).unwrap_or(bytes.len());
        let trimmed = match bytes[..end].iter().rposition(|&b| b != b' ') {
            Some(i) => &bytes[..=i],
            None => &[][..],
        };
        core::str::from_utf8(trimmed).unwrap_or("")
    }
}

impl core::fmt::Display for MonitorString {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl core::ops::Deref for MonitorString {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}
