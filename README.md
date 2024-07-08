# Simple coordinate frame conversions

This crate aims at supporting simple conversions between different standard and non-standard
coordinate frames. One potential use-case is in prototyping IMU sensor data where multiple
inertial or field sensors may be mounted in different orientations. These can then be expressed
in terms of coordinate frames such as `EastNorthUp` and trivially converted
to whatever basis you prefer, for example `NorthEastDown`.
