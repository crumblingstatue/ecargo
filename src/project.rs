use {
    crate::app::{LoadSend, LoadStage},
    cargo_metadata::{camino::Utf8PathBuf, CargoOpt, DependencyKind, MetadataCommand, Package},
    cargo_platform::Platform,
    slotmap::{new_key_type, SlotMap},
    std::{collections::HashMap, path::Path},
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
    pub enabled_features: Vec<String>,
    pub manifest_dir: Utf8PathBuf,
    pub readme_path: Option<Utf8PathBuf>,
    pub changelog_path: Option<Utf8PathBuf>,
}

pub type PkgSlotMap = SlotMap<PkgKey, Pkg>;

pub struct Project {
    pub packages: PkgSlotMap,
    pub root: Option<PkgKey>,
    pub license_map: HashMap<String, Vec<PkgKey>>,
}

new_key_type! {
    pub struct PkgKey;
}

impl Project {
    pub fn load(path: &Path, args: &crate::Args, sender: LoadSend) -> anyhow::Result<()> {
        let mut cmd = MetadataCommand::new();
        cmd.manifest_path(path.join("Cargo.toml"));
        if args.no_default_features {
            cmd.features(CargoOpt::NoDefaultFeatures);
        }
        if !args.features.is_empty() {
            cmd.features(CargoOpt::SomeFeatures(args.features.to_owned()));
        }
        if args.no_deps {
            cmd.no_deps();
        }
        sender.send(LoadStage::MetadataQuery)?;
        let metadata = cmd.exec()?;
        sender.send(LoadStage::PkgInfoCollect)?;
        let mut packages = SlotMap::with_key();
        let mut pkgid_key_mappings = HashMap::new();
        for package in &metadata.packages {
            packages.insert_with_key(|key| {
                pkgid_key_mappings.insert(package.id.clone(), key);
                let manifest_dir = package.manifest_path.parent().unwrap().to_owned();
                let readme_path = manifest_dir.join("README.md");
                let changelog_path = manifest_dir.join("CHANGELOG.md");
                Pkg {
                    cm_pkg: package.clone(),
                    key,
                    dependents: Vec::new(),
                    dependencies: Vec::new(),
                    enabled_features: Vec::new(),
                    manifest_dir,
                    readme_path: readme_path.exists().then_some(readme_path),
                    changelog_path: changelog_path.exists().then_some(changelog_path),
                }
            });
        }
        sender.send(LoadStage::Resolve)?;
        if let Some(resolve) = metadata.resolve.as_ref() {
            for node in &resolve.nodes {
                let pkg_key = pkgid_key_mappings[&node.id];
                packages[pkg_key].enabled_features.clone_from(&node.features);
            }
        }
        // Collect dependents
        sender.send(LoadStage::GenDepGraph)?;
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
        sender.send(LoadStage::Finished(Project {
            packages,
            root,
            license_map: HashMap::new(),
        }))?;
        Ok(())
    }
}

pub fn dep_matches_pkg(dep: &cargo_metadata::Dependency, pkg: &Pkg) -> bool {
    pkg.cm_pkg.name == dep.name && dep.req.matches(&pkg.cm_pkg.version)
}

// When I made this I didn't realize `cargo_metadata` supplied this information through `resolve`,
// but I don't feel like rewriting it right now.
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
