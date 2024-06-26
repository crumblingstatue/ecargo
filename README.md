# Ecargo

Cargo metadata viewer using egui.

Every time you add a dependency, Cargo dumps a huge amount of data into your home directory.
Wouldn't it be nice to be able to utilize this data to get a better overview of your
dependencies? This is what Ecargo sets out to accomplish.

## Features

### Filterable package list
![image](https://github.com/crumblingstatue/ecargo/assets/1521976/49379563-bd33-4920-8d94-90ce02dbc162)

Lists all the packages related to your crate. The filter searches the package name, description,
and keywords.

### Clickable links
![image](https://github.com/crumblingstatue/ecargo/assets/1521976/a480b16d-9dd5-48fe-8e99-e4945ad4a263)

All the relevant links, without needing to hop between websites to get the links you want.

### Dependency info
![image](https://github.com/crumblingstatue/ecargo/assets/1521976/8f984c2a-6505-4fc0-a7b5-a789b7c56a5b)

Get a good idea for how a package fits into your dependency chain, including:
- List of dependencies for each package
- What features are enabled?
- Which packages depend on this package?


### Glorious crates.io theme
What more do you need?

Don't worry, you can also use the vanilla dark and light egui themes.

## Installation
`cargo install ecargo`

There are no pre-built artifacts at the moment, but that may change.

## Usage
Ecargo requires a path to a cargo project to work with, so you need to run `ecargo` in
one of the following two ways:
1. In the working directory of a cargo project, just run `ecargo`
2. You can give `ecargo` a path to a cargo project: `ecargo /path/to/my/project` 

## Credits
All the heavy lifting is done by the [cargo-metadata](https://github.com/oli-obk/cargo_metadata) and [eframe](https://github.com/emilk/egui) crates (and their dependencies, of course).