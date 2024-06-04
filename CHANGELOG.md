# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added

- Readme viewer: View the readmes of your dependencies, if they exist locally.

  It can also be used to view changelogs.

- The preferred theme is now saved so you don't have to set it every time you open ecargo.

- Open in terminal button. Opens the package's working directory in the terminal of your choice.

- `View Cargo.toml.orig` button. Allows you to view the original `Cargo.toml` file of a package.

- `--no-default-features` and `--features` command line flags to configure feature set to resolve.

- `--no-deps` flag to skip resolving dependencies.

- `--version` command line flag

### Changed

- In the main view, the link containing the source folder location was removed in favor of a
  folder button that shows the location when hovered, and opens it when clicked.

- The folder icon in the sidebar now opens the source folder of the package, instead of
  focusing the package in the main view.

  There is an additional eye icon now which focuses the package in the main view.

- The close button on the sidebar's right side got replaced by a sidebar toggle button
  in the top panel's right side.

- The package list now only shows the first line of a package description.
  You can hover the description to get the full text, or click the package to open the
  side bar with the detailed view.

### Fixed

- crates.io theme: Make selected text more legible (higher color contrast)
- Allow packages to be explored even when there is no root package detected in a workspace.
- Very bad performance on package list tab, if there are a lot of packages.

  Now only the visible packages are rendered, vastly improving performance.

## [0.1.0] - 2024-04-25
Initial release
