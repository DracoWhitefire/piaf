#[derive(Debug, Clone, PartialEq)]
pub enum EdidWarning {
    UnknownExtension(u8),
    DescriptorParseFailed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdidError {
    InvalidLength,
    InvalidHeader,
    ChecksumMismatch,
}
