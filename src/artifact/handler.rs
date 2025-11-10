/// Artifact handler interface
pub trait ArtifactHandler {
    /// Get the directory extension for this artifact type
    fn directory_extension(&self) -> &str;

    /// Get the extension for this artifact type
    fn extension(&self) -> &str;

    /// Get the classifier for this artifact type
    fn classifier(&self) -> Option<&str> {
        None
    }

    /// Check if this handler can handle the given packaging type
    fn handles(&self, packaging: &str) -> bool;
}

/// Default artifact handler
pub struct DefaultArtifactHandler {
    extension: String,
}

impl DefaultArtifactHandler {
    pub fn new(extension: impl Into<String>) -> Self {
        Self {
            extension: extension.into(),
        }
    }
}

impl ArtifactHandler for DefaultArtifactHandler {
    fn directory_extension(&self) -> &str {
        &self.extension
    }

    fn extension(&self) -> &str {
        &self.extension
    }

    fn handles(&self, packaging: &str) -> bool {
        packaging == self.extension
    }
}

