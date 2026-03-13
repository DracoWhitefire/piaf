/// Manufacture date or model year, decoded from EDID base block bytes 16–17.
///
/// | Byte 16 | Meaning                                              |
/// |---------|------------------------------------------------------|
/// | `0x00`  | Week unspecified; byte 17 is the manufacture year.  |
/// | `0x01`–`0x36` | Week of manufacture (1–54).               |
/// | `0xFF`  | Byte 17 is a model year, not a manufacture year.    |
///
/// Year is encoded as `byte_17 + 1990`.
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
            w    => Self::Manufactured { week: Some(w), year },
        }
    }
}
