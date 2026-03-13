/// Video interface type, decoded from EDID base block byte `0x14` bits 3–0.
///
/// Only valid for digital input displays. `None` is used for the undefined (0x0)
/// and reserved (0x6–0xF) values.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoInterface {
    /// DVI interface.
    Dvi,
    /// HDMI-a interface.
    HdmiA,
    /// HDMI-b interface.
    HdmiB,
    /// MDDI (Mobile Display Digital Interface).
    Mddi,
    /// DisplayPort interface.
    DisplayPort,
}

impl VideoInterface {
    /// Decodes bits 3–0 of EDID byte `0x14` into a `VideoInterface`.
    ///
    /// Returns `None` for the undefined (0x0) and reserved (0x6–0xF) values.
    pub(crate) fn from_edid_bits(bits: u8) -> Option<Self> {
        match bits & 0x0F {
            0x1 => Some(Self::Dvi),
            0x2 => Some(Self::HdmiA),
            0x3 => Some(Self::HdmiB),
            0x4 => Some(Self::Mddi),
            0x5 => Some(Self::DisplayPort),
            _ => None, // 0x0 = undefined, 0x6-0xF = reserved
        }
    }
}
