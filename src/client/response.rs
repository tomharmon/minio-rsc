use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ObjectStat {
    pub(crate) bucket_name: String,
    pub(crate) object_name: String,
    pub(crate) last_modified: String,
    pub(crate) etag: String,
    pub(crate) content_type: String,
    pub(crate) version_id: String,
    pub(crate) size: usize,
    pub(crate) metadata: HashMap<String, String>,
}

impl ObjectStat {
    pub fn bucket_name(&self) -> &str {
        self.bucket_name.as_ref()
    }

    pub fn object_name(&self) -> &str {
        self.object_name.as_ref()
    }

    pub fn last_modified(&self) -> &str {
        self.last_modified.as_ref()
    }

    pub fn etag(&self) -> &str {
        self.etag.as_ref()
    }

    pub fn content_type(&self) -> &str {
        self.content_type.as_ref()
    }

    pub fn version_id(&self) -> &str {
        self.version_id.as_ref()
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}
