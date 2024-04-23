use {
    cargo_metadata::{Metadata, MetadataCommand, Package},
    slotmap::{new_key_type, SlotMap},
    std::path::Path,
};

pub struct Pkg {
    pub cm_pkg: Package,
    pub key: PkgKey,
}

pub struct Project {
    pub metadata: Metadata,
    pub packages: SlotMap<PkgKey, Pkg>,
    pub root: Option<PkgKey>,
}

new_key_type! {
    pub struct PkgKey;
}

impl Project {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let metadata = MetadataCommand::new()
            .manifest_path(path.join("Cargo.toml"))
            .exec()?;
        let mut packages = SlotMap::with_key();
        for package in &metadata.packages {
            packages.insert_with_key(|key| Pkg {
                cm_pkg: package.clone(),
                key,
            });
        }
        let root;
        match metadata.root_package() {
            Some(pkg) => {
                if let Some(pkg) = packages.values().find(|pkg_inner| pkg_inner.cm_pkg == *pkg) {
                    root = Some(pkg.key);
                } else {
                    root = None;
                }
            }
            None => root = None,
        }
        Ok(Project {
            metadata,
            packages,
            root,
        })
    }
}
