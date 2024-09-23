pub mod markdown;
pub mod package_list;
pub mod view_single;

#[derive(Default, PartialEq, Eq)]
pub enum Tab {
    #[default]
    ViewSingle,
    PackageList,
    Markdown,
}
