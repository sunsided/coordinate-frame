# Changelog

All notable changes to this project will be documented in this file.
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Added the `construct_frame` function to construct a coordinate frame from values and a `CoordinateFrameType` variant.
- Added the `new_from` function to a coordinate frame to simplify access to `construct_frame`.

## [0.5.0] - 2024-07-14

[0.5.0]: https://github.com/sunsided/coordinate-frame/releases/tag/v0.5.0

### Added

- The `COORDINATE_FRAME` constant is now public.
- The documentation was improved with some visualizations about the coordinate system layouts.
- The `coordinate_frame` is now available without the `CoordinateFrame` trait.
- Added the `map` function to apply a transformation to each value of the coordinate system.

### Changed

- The `x`, `y` and `z` functions of each type are now linked up with their respective directions
  in the documentation
- The base vector functions now return the primary axes in their own coordinate frame.

## [0.4.0] - 2024-07-13

[0.4.0]: https://github.com/sunsided/coordinate-frame/releases/tag/v0.4.0

### Added

- Added the `flip_frame` function to switch a coordinate frame into its opposite, e.g. going
  from `NorthEastDown` to `SouthWestUp`.
- Added a `From<[T; 3]>` conversion to the types.
- Added a `from_slice` constructor function to the types.

### Changed

- The coordinate frames are now explicitly `repr(C)`.
