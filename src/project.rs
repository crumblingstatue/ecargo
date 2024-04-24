use {
    cargo_metadata::{DependencyKind, Metadata, MetadataCommand, Package},
    cargo_platform::Platform,
    slotmap::{new_key_type, SlotMap},
    std::path::Path,
};

pub struct DepLink {
    pub pkg_key: PkgKey,
    pub kind: DependencyKind,
    pub target: Option<Platform>,
}

pub struct Pkg {
    pub cm_pkg: Package,
    pub key: PkgKey,
    pub dependents: Vec<DepLink>,
    pub dependencies: Vec<DepLink>,
}

pub type PkgSlotMap = SlotMap<PkgKey, Pkg>;

pub struct Project {
    pub metadata: Metadata,
    pub packages: PkgSlotMap,
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
                dependents: Vec::new(),
                dependencies: Vec::new(),
            });
        }
        // Collect dependents
        gen_dep_graph_info(&mut packages);
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

pub fn dep_matches_pkg(dep: &cargo_metadata::Dependency, pkg: &Pkg) -> bool {
    pkg.cm_pkg.name == dep.name && dep.req.matches(&pkg.cm_pkg.version)
}

fn gen_dep_graph_info(pkgs: &mut PkgSlotMap) {
    let keys: Vec<PkgKey> = pkgs.keys().collect();
    for a in &keys {
        for b in &keys {
            // Don't do anything for self
            if a == b {
                continue;
            }
            let [pkg_a, pkg_b] = pkgs.get_disjoint_mut([*a, *b]).unwrap();
            for dep in &pkg_a.cm_pkg.dependencies {
                if dep_matches_pkg(dep, pkg_b) {
                    pkg_a.dependencies.push(DepLink {
                        pkg_key: *b,
                        kind: dep.kind,
                        target: dep.target.clone(),
                    });
                    pkg_b.dependents.push(DepLink {
                        pkg_key: *a,
                        kind: dep.kind,
                        target: dep.target.clone(),
                    });
                }
            }
        }
    }
    for pkg in pkgs.values_mut() {
        pkg.dependencies.dedup_by_key(|link| link.pkg_key);
        pkg.dependents.dedup_by_key(|link| link.pkg_key);
    }
}
