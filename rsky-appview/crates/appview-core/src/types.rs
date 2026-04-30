/// A parsed AT Protocol URI: `at://<did>/<collection>/<rkey>`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AtUri(String);

impl AtUri {
    /// Construct an AtUri from a raw string without validation.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn after_scheme(&self) -> &str {
        self.0.strip_prefix("at://").unwrap_or(&self.0)
    }

    /// Returns the DID portion (the authority segment).
    pub fn did(&self) -> &str {
        let s = self.after_scheme();
        match s.find('/') {
            Some(idx) => &s[..idx],
            None => s,
        }
    }

    /// Returns the collection NSID portion, or an empty string if absent.
    pub fn collection(&self) -> &str {
        let s = self.after_scheme();
        let after_did = match s.find('/') {
            Some(idx) => &s[idx + 1..],
            None => return "",
        };
        match after_did.find('/') {
            Some(idx) => &after_did[..idx],
            None => after_did,
        }
    }

    /// Returns the rkey portion, or an empty string if absent.
    pub fn rkey(&self) -> &str {
        let s = self.after_scheme();
        let after_did = match s.find('/') {
            Some(idx) => &s[idx + 1..],
            None => return "",
        };
        match after_did.find('/') {
            Some(idx) => &after_did[idx + 1..],
            None => "",
        }
    }
}

impl std::fmt::Display for AtUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for AtUri {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for AtUri {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// A W3C Decentralized Identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Did(String);

impl Did {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Did {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for Did {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Did {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// A Content Identifier (CID) string.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cid(String);

impl Cid {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Cid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for Cid {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Cid {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

/// An opaque pagination cursor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cursor(String);

impl Cursor {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Cursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for Cursor {
    fn from(s: String) -> Self {
        Self(s)
    }
}
