# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2023-03-21

### Fixed

- `poison` incorrectly returned `Uwildmat::Poison` when text and pattern were empty

### Changed

- Moved public functions to `uwildmat::{regular, simple, poison}`

### Added

- Implement `From<bool>` for `Uwildmat`

## [0.2.0] - 2023-03-12

### Added

- `Into<u8>` implemenation for `Uwildmat`

## [0.1.0] - 2023-03-12

- First release

[unreleased]: https://github.com/olivierlacan/keep-a-changelog/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/olivierlacan/keep-a-changelog/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/olivierlacan/keep-a-changelog/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/olivierlacan/keep-a-changelog/releases/tag/v0.1.0
