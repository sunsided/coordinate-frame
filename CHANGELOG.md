# Changelog

All notable changes to this project will be documented in this file.
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Changed

- The `x`, `y` and `z` functions of each type are now linked up with their respective directions
  in the documentation
- The base vector functions are now marked deprecated as they are ambiguous with respect to their reference frame.

## [0.4.0] - 2024-07-13

[0.4.0]: https://github.com/sunsided/coordinate-frame/releases/tag/v0.4.0

### Added

- Added the `flip_frame` function to switch a coordinate frame into its opposite, e.g. going
  from `NorthEastDown` to `SouthWestUp`.
- Added a `From<[T; 3]>` conversion to the types.
- Added a `from_slice` constructor function to the types.

### Changed

- The coordinate frames are now explicitly `repr(C)`.
