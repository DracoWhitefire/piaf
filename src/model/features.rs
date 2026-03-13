bitflags::bitflags! {
    /// Boolean capability flags from EDID base block byte `0x18` (Display Feature Support).
    ///
    /// Bits 4–3 of this byte encode a color type / supported encoding field whose meaning
    /// differs for digital and analog displays. That field is not represented here — it
    /// requires a dedicated enum and will be added alongside color encoding support.
    ///
    /// | Bit | Mask   | Meaning                                                      |
    /// |-----|--------|--------------------------------------------------------------|
    /// | 7   | `0x80` | DPMS standby supported                                      |
    /// | 6   | `0x40` | DPMS suspend supported                                       |
    /// | 5   | `0x20` | DPMS active-off supported                                    |
    /// | 2   | `0x04` | sRGB is the default color space                             |
    /// | 1   | `0x02` | Preferred timing includes native pixel format and rate       |
    /// | 0   | `0x01` | Continuous timings supported (GTF or CVT, EDID 1.4+)         |
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DisplayFeatureFlags: u8 {
        /// DPMS standby mode is supported.
        const DPMS_STANDBY        = 0x80;
        /// DPMS suspend mode is supported.
        const DPMS_SUSPEND        = 0x40;
        /// DPMS active-off mode is supported.
        const DPMS_ACTIVE_OFF     = 0x20;
        /// sRGB is the default color space for this display.
        const SRGB                = 0x04;
        /// The preferred timing mode (first DTD) includes the native pixel format
        /// and preferred refresh rate.
        const PREFERRED_TIMING    = 0x02;
        /// Continuous timings are supported via GTF or CVT (EDID 1.4+).
        const CONTINUOUS_TIMINGS  = 0x01;
    }
}
