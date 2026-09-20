use std::error::Error;
use std::fmt;

/// A stable, platform-supplied identity for a user-recognizable application.
///
/// One value represents the application after the platform adapter has grouped
/// child processes and audio sessions. Display names and process IDs are not valid
/// substitutes for this identity.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationId(String);

impl ApplicationId {
    /// Creates an owned identity. Empty identities are rejected at the platform seam.
    pub fn new(value: impl Into<String>) -> Result<Self, IdentifierError> {
        let value = value.into();
        if value.is_empty() {
            return Err(IdentifierError);
        }
        Ok(Self(value))
    }

    /// Returns the platform identity without transferring ownership.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A stable, platform-supplied identity for one exact output device.
///
/// A friendly name, manufacturer, or model must not be used as this value because
/// route restoration requires exact identity equality.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct OutputId(String);

impl OutputId {
    /// Creates an owned identity. Empty identities are rejected at the platform seam.
    pub fn new(value: impl Into<String>) -> Result<Self, IdentifierError> {
        let value = value.into();
        if value.is_empty() {
            return Err(IdentifierError);
        }
        Ok(Self(value))
    }

    /// Returns the platform identity without transferring ownership.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Returned when a platform adapter supplies an empty stable identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IdentifierError;

impl fmt::Display for IdentifierError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a stable identity cannot be empty")
    }
}

impl Error for IdentifierError {}
