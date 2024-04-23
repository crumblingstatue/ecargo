use {
    cargo_metadata::{Metadata, MetadataCommand},
    std::path::Path,
};

pub struct Project {
    pub metadata: Metadata,
}

impl Project {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let metadata = MetadataCommand::new()
            .manifest_path(path.join("Cargo.toml"))
            .exec()?;
        Ok(Project { metadata })
    }
}
