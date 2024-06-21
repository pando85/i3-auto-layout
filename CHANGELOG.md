# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.3.3](https://github.com/pando85/i3-auto-layout/tree/v0.3.3) - 2024-06-21

### Build

* Update Rust crate flexi_logger to 0.28.0
* Update wagoid/commitlint-github-action action to v6.0.1
* Update Rust crate flexi_logger to v0.28.4
* Update Rust crate tokio to v1.38.0
* Update Rust crate anyhow to v1.0.86

### CI

* Add automerge for renovate patch versions

### Fixed

* Change flexi logger to UTC

## [v0.3.2](https://github.com/pando85/i3-auto-layout/tree/v0.3.2) - 2024-05-15

### Added

- add timestamp to log

### Fixed

- Recreate channel when channel closed

### Build

- Update KSXGitHub/github-actions-deploy-aur action to v2.7.1

### Documentation

- Update release test copy dir

## [v0.3.1](https://github.com/pando85/i3-auto-layout/tree/v0.3.1) - 2024-05-09

### CI

- Remove mac release

### Documentation

- Fix release workflow order

### Fixed

- Parse version correctly in aur action

## [v0.3.0](https://github.com/pando85/i3-auto-layout/tree/v0.3.0) - 2024-05-09

### Core

- Add CI, doc and update dependencies
