#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::capabilities::DisplayCapabilities;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::diagnostics::EdidWarning;
#[cfg(any(feature = "alloc", feature = "std"))]
use crate::model::prelude::{Box, String, Vec};

/// Processes a single 128-byte EDID block and populates [`DisplayCapabilities`].
///
/// Implement this trait to add support for extension block formats or to customise
/// base block parsing. Register handlers via [`ExtensionLibrary`].
#[cfg(any(feature = "alloc", feature = "std"))]
pub trait ExtensionHandler: core::fmt::Debug {
    /// Inspect `block` and update `caps` accordingly.
    ///
    /// Push non-fatal issues to `warnings` rather than panicking or discarding silently.
    fn process(
        &self,
        block: &[u8; 128],
        caps: &mut DisplayCapabilities,
        warnings: &mut Vec<EdidWarning>,
    );
}

/// Registration entry for an EDID extension block type.
#[cfg(any(feature = "alloc", feature = "std"))]
pub struct ExtensionMetadata {
    /// The tag byte that identifies this extension block format (e.g. `0x02` for CEA-861).
    pub tag: u8,
    /// Human-readable name used in diagnostics.
    pub display_name: String,
    /// Optional handler that processes blocks with this tag.
    /// `None` means the tag is recognised but no processing is performed.
    pub handler: Option<Box<dyn ExtensionHandler>>,
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl core::fmt::Debug for ExtensionMetadata {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ExtensionMetadata")
            .field("tag", &self.tag)
            .field("display_name", &self.display_name)
            .field("has_handler", &self.handler.is_some())
            .finish()
    }
}

/// Minimal registration entry for an EDID extension tag (`no_std` build).
#[cfg(not(any(feature = "alloc", feature = "std")))]
#[derive(Debug, Clone, PartialEq)]
pub struct ExtensionMetadata {
    /// The tag byte that identifies this extension block format.
    pub tag: u8,
}

/// Implemented by types that can tell the parser which extension tags are known.
/// Implemented for both [`ExtensionTagRegistry`] and [`ExtensionLibrary`], so either
/// can be passed directly to [`parse_edid`][crate::parse_edid].
pub trait KnownExtensions {
    /// Returns `true` if `tag` identifies a known extension block type.
    fn is_known(&self, tag: u8) -> bool;
}

/// A lightweight set of known extension tag bytes.
///
/// Use this when you need to pass a tag list to [`parse_edid`][crate::parse_edid] without
/// setting up a full [`ExtensionLibrary`]. If you are using `ExtensionLibrary`, you can pass it
/// directly to `parse_edid` instead — it implements [`KnownExtensions`] too.
#[cfg(any(feature = "alloc", feature = "std"))]
pub struct ExtensionTagRegistry {
    /// The registered tag bytes.
    pub known_tags: Vec<u8>,
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl Default for ExtensionTagRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl ExtensionTagRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            known_tags: Vec::new(),
        }
    }

    /// Adds `tag` to the set of known tags. Duplicate registrations are silently ignored.
    pub fn register(&mut self, tag: u8) {
        if !self.known_tags.contains(&tag) {
            self.known_tags.push(tag);
        }
    }

    /// Returns `true` if `tag` has been registered.
    pub fn is_known(&self, tag: u8) -> bool {
        self.known_tags.contains(&tag)
    }
}

/// A fixed-capacity set of known extension tag bytes (`no_std` build, capacity 16).
#[cfg(not(any(feature = "alloc", feature = "std")))]
pub struct ExtensionTagRegistry {
    tags: [u8; 16],
    len: usize,
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl Default for ExtensionTagRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(any(feature = "alloc", feature = "std")))]
impl ExtensionTagRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            tags: [0u8; 16],
            len: 0,
        }
    }

    /// Adds `tag` to the set of known tags.
    /// Duplicate registrations and overflow (capacity 16) are silently ignored.
    pub fn register(&mut self, tag: u8) {
        if self.len < 16 && !self.is_known(tag) {
            self.tags[self.len] = tag;
            self.len += 1;
        }
    }

    /// Returns `true` if `tag` has been registered.
    pub fn is_known(&self, tag: u8) -> bool {
        self.tags[..self.len].contains(&tag)
    }
}

impl KnownExtensions for ExtensionTagRegistry {
    fn is_known(&self, tag: u8) -> bool {
        ExtensionTagRegistry::is_known(self, tag)
    }
}

/// A registry of extension block handlers and base block handlers.
///
/// Pass to [`parse_edid`][crate::parse_edid] (it implements [`KnownExtensions`]) and to
/// [`capabilities_from_edid`][crate::capabilities_from_edid] to drive capability extraction.
///
/// Use `ExtensionLibrary::with_standard_handlers` (available when the `std` or `alloc` feature
/// is enabled) to get a library pre-loaded with the built-in base block and CEA-861 handlers.
pub struct ExtensionLibrary {
    /// Handlers run against the base block, in registration order.
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub base_handlers: Vec<Box<dyn ExtensionHandler>>,
    /// Registered extension block types with optional handlers.
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub extensions: Vec<ExtensionMetadata>,
}

impl Default for ExtensionLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtensionLibrary {
    /// Creates an empty library with no handlers or registered extensions.
    pub fn new() -> Self {
        Self {
            #[cfg(any(feature = "alloc", feature = "std"))]
            base_handlers: Vec::new(),
            #[cfg(any(feature = "alloc", feature = "std"))]
            extensions: Vec::new(),
        }
    }

    /// Appends a handler that will be run against the base block during capability extraction.
    ///
    /// Handlers are called in registration order. Multiple base handlers may be registered.
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub fn add_base_handler<H: ExtensionHandler + 'static>(&mut self, handler: H) {
        self.base_handlers.push(Box::new(handler));
    }

    /// Creates a library with CEA-861 (`0x02`) and DisplayID (`0x70`) registered as known
    /// tags but without any handlers attached. Handlers can be added with [`register`][Self::register].
    ///
    /// Prefer `ExtensionLibrary::with_standard_handlers` unless you need to customise the handlers.
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub fn with_standard_extensions() -> Self {
        let mut lib = Self::new();
        lib.register(ExtensionMetadata {
            tag: 0x02,
            display_name: String::from("CEA-861"),
            handler: None, // Will be set by caller or library user
        });
        lib.register(ExtensionMetadata {
            tag: 0x70,
            display_name: String::from("DisplayID"),
            handler: None,
        });
        lib.register(ExtensionMetadata {
            tag: 0xF0,
            display_name: String::from("Block Map"),
            handler: None, // Contents not needed; blocks are discovered sequentially
        });
        lib
    }

    /// Registers an extension block type. Duplicate tags (same `metadata.tag`) are ignored.
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub fn register(&mut self, metadata: ExtensionMetadata) {
        if !self.extensions.iter().any(|ext| ext.tag == metadata.tag) {
            self.extensions.push(metadata);
        }
    }

    /// Returns an [`ExtensionTagRegistry`] containing all tags registered in this library.
    ///
    /// Useful when you need a [`KnownExtensions`] value without handing ownership of the
    /// full library to the parser.
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub fn export_tags(&self) -> ExtensionTagRegistry {
        let mut known_tags = Vec::new();
        for ext in &self.extensions {
            known_tags.push(ext.tag);
        }
        ExtensionTagRegistry { known_tags }
    }

    /// Returns an empty [`ExtensionTagRegistry`] (`no_std` build — tag export not supported).
    #[cfg(not(any(feature = "alloc", feature = "std")))]
    pub fn export_tags(&self) -> ExtensionTagRegistry {
        ExtensionTagRegistry::new()
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl KnownExtensions for ExtensionLibrary {
    fn is_known(&self, tag: u8) -> bool {
        self.extensions.iter().any(|e| e.tag == tag)
    }
}
