mod licenses;
mod markdown;
mod package_list;
mod view_single;

pub(crate) use self::{
    licenses::licenses_ui, markdown::markdown_ui, package_list::package_list_ui,
    view_single::view_single_ui,
};

#[derive(Default, PartialEq)]
pub enum Tab {
    #[default]
    ViewSingle,
    PackageList,
    Markdown,
    Licenses,
}
