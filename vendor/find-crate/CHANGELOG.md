# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org).

<!--
Note: In this file, do not use the hard wrap in the middle of a sentence for compatibility with GitHub comment style markdown rendering.
-->

## [Unreleased]

## [0.6.3] - 2021-01-05

- Exclude unneeded files from crates.io.

## [0.6.2] - 2020-12-29

- Documentation improvements.

## [0.6.1] - 2020-09-07

- Documentation improvements.

## [0.6.0] - 2020-08-27

- [Add `Manifest::crate_package`.](https://github.com/taiki-e/find-crate/pull/12)

- Make `Error` non-exhaustive.

## [0.5.0] - 2019-09-29

- Made `Manifest::dependencies` and `Package::{name, varsion}` fields public.

- Added support for `target.cfg.dependencies`.

- Added `Dependencies` enum to manage the kind of dependencies to be searched.

- Removed `Manifest::lock()` and `ManifestLock`.

- Removed some variant and field form `Error`.

- Removed `DEFAULT_DEPENDENCIES`.

## [0.4.0] - 2019-06-16

- Transition to Rust 2018. With this change, the minimum required version will go up to Rust 1.31.

- Updated minimum `toml` version to 0.5.0.

## [0.3.0] - 2019-02-21

- Removed version dependent behavior.

- Documentation improvements.

## [0.2.0] - 2019-02-13

- Supported Rust 1.15.

## [0.1.2] - 2019-02-13

- Implemented `PartialEq` and `Eq` for `Package`.

## [0.1.1] - 2019-02-13

- Documentation improvements.

## [0.1.0] - 2019-02-13

Initial release

[Unreleased]: https://github.com/taiki-e/find-crate/compare/v0.6.3...HEAD
[0.6.3]: https://github.com/taiki-e/find-crate/compare/v0.6.2...v0.6.3
[0.6.2]: https://github.com/taiki-e/find-crate/compare/v0.6.1...v0.6.2
[0.6.1]: https://github.com/taiki-e/find-crate/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/taiki-e/find-crate/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/taiki-e/find-crate/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/taiki-e/find-crate/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/taiki-e/find-crate/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/taiki-e/find-crate/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/taiki-e/find-crate/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/taiki-e/find-crate/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/taiki-e/find-crate/releases/tag/v0.1.0
