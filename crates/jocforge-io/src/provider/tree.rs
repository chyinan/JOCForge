// pattern: Imperative Shell
//! Read-only view of one approved provider package tree.
//!
//! Natively the tree is a directory: every read resolves `root/relative`, proves the resolved
//! path stays inside the root, and reads those bytes. The browser target has no directory to
//! open, so the same relative names resolve against the approved bytes embedded in the encode
//! pipeline. Both flavours answer the same `read(relative)` calls, which keeps every loader,
//! manifest identity check and provider receipt identical on both targets.

use std::{
    io,
    path::{Component, Path},
    sync::Arc,
};

/// One embedded provider member: the approved relative name and its exact bytes.
pub type ProviderMember = (&'static str, &'static [u8]);

/// Failure to read one approved provider member.
#[derive(Debug)]
pub enum ProviderReadError {
    /// The relative name escapes the package root.
    EscapesRoot,
    /// The member is absent from the tree.
    Missing,
    /// The backing store reported an I/O failure.
    Io(io::Error),
}

impl ProviderReadError {
    #[cfg(not(target_arch = "wasm32"))]
    fn from_io(error: io::Error) -> Self {
        if error.kind() == io::ErrorKind::NotFound {
            Self::Missing
        } else {
            Self::Io(error)
        }
    }

    /// Returns the underlying I/O error for callers that report a read failure.
    pub fn into_io_error(self, relative: &'static str) -> io::Error {
        match self {
            Self::EscapesRoot => io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("provider path escapes the package root: {relative}"),
            ),
            Self::Missing => io::Error::new(
                io::ErrorKind::NotFound,
                format!("provider member is absent: {relative}"),
            ),
            Self::Io(error) => error,
        }
    }
}

/// A caller-selected approved provider package.
#[derive(Clone, Copy, Debug)]
pub struct ProviderTree<'a> {
    root: &'a Path,
    prefix: &'static str,
    #[cfg(target_arch = "wasm32")]
    members: &'a [ProviderMember],
}

impl<'a> ProviderTree<'a> {
    /// A package tree rooted at one caller-selected directory.
    pub const fn directory(root: &'a Path) -> Self {
        Self {
            root,
            prefix: "",
            #[cfg(target_arch = "wasm32")]
            members: &[],
        }
    }

    /// A package tree resolved against approved bytes embedded in the calling binary.
    // Not `const`: `Path::new` is not yet stable in a const context, and the synthetic root is a
    // `&Path`.
    #[cfg(target_arch = "wasm32")]
    pub fn embedded(members: &'a [ProviderMember]) -> Self {
        Self {
            root: Path::new("/jocforge-approved-provider-bundle"),
            prefix: "",
            members,
        }
    }

    /// A one-level sub-package below this tree, such as one approved supplement bundle.
    #[must_use]
    pub const fn child(self, prefix: &'static str) -> Self {
        Self { prefix, ..self }
    }

    /// The directory this tree was built from, or the synthetic embedded root.
    pub const fn root(&self) -> &'a Path {
        self.root
    }

    /// The one-level sub-package prefix, empty for the package root itself.
    pub const fn prefix(&self) -> &'static str {
        self.prefix
    }

    /// The display path recorded in provider evidence for one relative member name.
    pub fn display(&self, relative: &'static str) -> String {
        self.root
            .join(self.prefix)
            .join(relative)
            .display()
            .to_string()
    }

    /// Reads one relative member after proving it cannot leave the package root.
    pub fn read(&self, relative: &'static str) -> Result<Arc<[u8]>, ProviderReadError> {
        let relative_path = Path::new(relative);
        if relative_path.is_absolute()
            || relative_path
                .components()
                .any(|component| matches!(component, Component::ParentDir))
        {
            return Err(ProviderReadError::EscapesRoot);
        }
        self.read_resolved(relative)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn read_resolved(&self, relative: &'static str) -> Result<Arc<[u8]>, ProviderReadError> {
        // The containment proof compares two resolved paths, so the root must be resolved first:
        // `canonicalize` returns a verbatim `\\?\` path on Windows and keeps no `..` components,
        // and a caller-selected root may carry either.
        let canonical_root = self
            .root
            .join(self.prefix)
            .canonicalize()
            .map_err(ProviderReadError::from_io)?;
        let resolved = canonical_root
            .join(relative)
            .canonicalize()
            .map_err(ProviderReadError::from_io)?;
        if !resolved.starts_with(&canonical_root) {
            return Err(ProviderReadError::EscapesRoot);
        }
        std::fs::read(resolved)
            .map(Arc::<[u8]>::from)
            .map_err(ProviderReadError::from_io)
    }

    #[cfg(target_arch = "wasm32")]
    fn read_resolved(&self, relative: &'static str) -> Result<Arc<[u8]>, ProviderReadError> {
        let key = if self.prefix.is_empty() {
            relative.to_owned()
        } else {
            format!("{}/{relative}", self.prefix)
        };
        // The embedding build script records the relative member name with the host's
        // separator, so the lookup compares a separator-normalized form.
        self.members
            .iter()
            .find(|(name, _)| name.replace('\\', "/") == key)
            .map(|(_, bytes)| Arc::<[u8]>::from(*bytes))
            .ok_or(ProviderReadError::Missing)
    }
}

impl<'a> From<&'a Path> for ProviderTree<'a> {
    fn from(root: &'a Path) -> Self {
        Self::directory(root)
    }
}

impl AsRef<Path> for ProviderTree<'_> {
    fn as_ref(&self) -> &Path {
        self.root
    }
}
