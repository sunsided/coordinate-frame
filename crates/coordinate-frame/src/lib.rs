//! # Simple coordinate frame conversions
//!
//! This crate aims at supporting simple conversions between different standard and non-standard
//! coordinate frames. One potential use-case is in prototyping IMU sensor data where multiple
//! inertial or field sensors may be mounted in different orientations. These can then be expressed
//! in terms of coordinate frames such as [`EastNorthUp`] and trivially converted
//! to whatever basis you prefer, for example [`NorthEastDown`].
//!
//! ## Example
//! ```
//! use coordinate_frame::{NorthEastDown, NorthEastUp};
//!
//! // Construct a coordinate in one reference frame.
//! let neu = NorthEastUp::new(1.0, 2.0, 3.0);
//! assert_eq!(neu.north(), 1.0);
//! assert_eq!(neu.east(), 2.0);
//! assert_eq!(neu.up(), 3.0);
//!
//! // Note that "non-native" axes are also available.
//! assert_eq!(neu.down(), -3.0);
//!
//! // You can transform it into a different frame.
//! let ned: NorthEastDown<_> = neu.into();
//! assert_eq!(ned.north(), 1.0);
//! assert_eq!(ned.east(), 2.0);
//! assert_eq!(ned.down(), -3.0);
//!
//! // Information is available as you'd expect.
//! assert_eq!(ned, &[1.0, 2.0, -3.0]);
//! assert_eq!(ned.x(), 1.0);
//! assert_eq!(ned.z(), -3.0);
//!
//! // Base vectors are also provided.
//! let axis = NorthEastDown::<f64>::z_axis();
//! assert_eq!(axis, [0.0, 0.0, -1.0]);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod traits;

use coordinate_frame_derive::CoordinateFrame;
pub use traits::*;

/// A coordinate frame type.
#[derive(CoordinateFrame, Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(u8)]
pub enum CoordinateFrameType {
    /// Common aerospace reference frame.
    /// See [`NorthEastDown`].
    #[default]
    NorthEastDown = 0,
    /// See [`NorthEastUp`].
    NorthEastUp = 1,
    /// See [`NorthWestDown`].
    NorthWestDown = 2,
    /// See [`NorthWestUp`].
    NorthWestUp = 3,
    /// See [`NorthDownEast`].
    NorthDownEast = 4,
    /// See [`NorthDownWest`].
    NorthDownWest = 5,
    /// See [`NorthUpEast`].
    NorthUpEast = 6,
    /// See [`NorthUpWest`].
    NorthUpWest = 7,
    /// See [`EastNorthDown`].
    EastNorthDown = 8,
    /// Common geography reference frame.
    /// See [`EastNorthUp`].
    EastNorthUp = 9,
    /// See [`EastSouthDown`].
    EastSouthDown = 10,
    /// See [`EastSouthUp`].
    EastSouthUp = 11,
    /// See [`EastDownNorth`].
    ///
    /// ## Example usage
    /// This is commonly a right-handed image-space reference frame with the origin in the top-left
    /// corner and `z` pointing into the screen, away from the viewer.
    EastDownNorth = 12,
    /// See [`EastDownSouth`].
    ///
    /// ## Example usage
    /// This is commonly a left-handed image-space reference frame with the origin in the top-left
    /// corner and `z` pointing out of the screen, toward the viewer.
    EastDownSouth = 13,
    /// See [`EastUpNorth`].
    EastUpNorth = 14,
    /// See [`EastUpSouth`].
    EastUpSouth = 15,
    /// See [`SouthEastDown`].
    SouthEastDown = 16,
    /// See [`SouthEastUp`].
    SouthEastUp = 17,
    /// See [`SouthWestDown`].
    SouthWestDown = 18,
    /// See [`SouthWestUp`].
    SouthWestUp = 19,
    /// See [`SouthDownEast`].
    SouthDownEast = 20,
    /// See [`SouthDownWest`].
    SouthDownWest = 21,
    /// See [`SouthUpEast`].
    SouthUpEast = 22,
    /// See [`SouthUpWest`].
    SouthUpWest = 23,
    /// See [`WestNorthDown`].
    WestNorthDown = 24,
    /// See [`WestNorthUp`].
    WestNorthUp = 25,
    /// See [`WestSouthDown`].
    WestSouthDown = 26,
    /// See [`WestSouthUp`].
    WestSouthUp = 27,
    /// See [`WestDownNorth`].
    WestDownNorth = 28,
    /// See [`WestDownSouth`].
    WestDownSouth = 29,
    /// See [`WestUpNorth`].
    WestUpNorth = 30,
    /// See [`WestUpSouth`].
    WestUpSouth = 31,
    /// See [`DownNorthEast`].
    DownNorthEast = 32,
    /// See [`DownNorthWest`].
    DownNorthWest = 33,
    /// See [`DownEastNorth`].
    DownEastNorth = 34,
    /// See [`DownEastSouth`].
    DownEastSouth = 35,
    /// See [`DownSouthEast`].
    DownSouthEast = 36,
    /// See [`DownSouthWest`].
    DownSouthWest = 37,
    /// See [`DownWestNorth`].
    DownWestNorth = 38,
    /// See [`DownWestSouth`].
    DownWestSouth = 39,
    /// See [`UpNorthEast`].
    UpNorthEast = 40,
    /// See [`UpNorthWest`].
    UpNorthWest = 41,
    /// See [`UpEastNorth`].
    UpEastNorth = 42,
    /// See [`UpEastSouth`].
    UpEastSouth = 43,
    /// See [`UpSouthEast`].
    UpSouthEast = 44,
    /// See [`UpSouthWest`].
    UpSouthWest = 45,
    /// See [`UpWestNorth`].
    UpWestNorth = 46,
    /// See [`UpWestSouth`].
    UpWestSouth = 47,
    /// An orientation represented by a rotation matrix.
    Other = 48,
    /// An undefined system.
    Undefined = 255,
}

#[derive(Debug)]
pub enum ParseCoordinateFrameError {
    /// An unknown enum variant was provided.
    UnknownVariant,
}

#[cfg(test)]
mod tests {
    use crate::{EastNorthUp, NorthEastDown, NorthEastUp, SouthWestUp};

    #[test]
    fn neu_to_ned() {
        let neu = NorthEastUp::new(0.0, 2.0, 3.0);
        let neu = neu.with_north(1.0);

        assert_eq!(neu.north(), 1.0);
        assert_eq!(neu.east(), 2.0);
        assert_eq!(neu.up(), 3.0);

        // Generated
        assert_eq!(neu.down(), -3.0);

        assert_eq!(neu.north_ref(), &1.0);
        assert_eq!(neu.east_ref(), &2.0);
        assert_eq!(neu.up_ref(), &3.0);

        let ned: NorthEastDown<_> = neu.into();
        assert_eq!(ned.north(), 1.0);
        assert_eq!(ned.east(), 2.0);
        assert_eq!(ned.down(), -3.0);

        assert_eq!(ned.0, [1.0, 2.0, -3.0]);
        assert_eq!(ned.x(), 1.0);
        assert_eq!(ned.z(), -3.0);

        let axis = NorthEastDown::<f64>::z_axis();
        assert_eq!(axis, [0.0, 0.0, -1.0]);
    }

    #[test]
    fn ned_to_enu() {
        let ned = NorthEastDown([1.0, 2.0, 3.0]);
        let enu: EastNorthUp<_> = ned.into();
        assert_eq!(enu.0, [2.0, 1.0, -3.0]);
    }

    #[test]
    fn flip() {
        let ned = NorthEastDown([1.0, 2.0, 3.0]);
        let swu: SouthWestUp<_> = ned.flip_frame();
        assert_eq!(swu.0, [-1.0, -2.0, -3.0]);
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_permutations() {
        const LATERAL: [&str; 2] = ["East", "West"];
        const LONGITUDINAL: [&str; 2] = ["North", "South"];
        const VERTICAL: [&str; 2] = ["Down", "Up"];
        const MUTUALLY_EXCLUSIVE: [[&str; 2]; 3] = [LATERAL, LONGITUDINAL, VERTICAL];
        const DIRECTIONS: [&str; 6] = ["North", "East", "South", "West", "Down", "Up"];
        let mut permutations = Vec::new();
        for x in DIRECTIONS.iter() {
            'y: for y in DIRECTIONS.iter() {
                for pair in MUTUALLY_EXCLUSIVE {
                    if pair.contains(y) && pair.contains(x) {
                        continue 'y;
                    }
                }

                let permutation = format!("{x}{y}");
                'z: for z in DIRECTIONS.iter() {
                    for pair in MUTUALLY_EXCLUSIVE {
                        if pair.contains(z) && (pair.contains(x) || pair.contains(y)) {
                            continue 'z;
                        }
                    }
                    permutations.push(format!("{permutation}{z}"));
                }
            }
        }
        // let str = permutations.join(",");
        assert_eq!(permutations.len(), 48);
    }

    #[test]
    #[cfg(feature = "nalgebra")]
    fn nalgebra_from_point3() {
        let ned = NorthEastDown::from(nalgebra::Point3::new(1.0, 2.0, 3.0)).to_enu();
        let point: nalgebra::Point3<_> = ned.into();
        assert_eq!(point.x, 2.0);
        assert_eq!(point.y, 1.0);
        assert_eq!(point.z, -3.0);
    }

    #[test]
    #[cfg(feature = "nalgebra")]
    fn nalgebra_from_vector() {
        let ned = NorthEastDown::from(nalgebra::Vector3::new(1.0, 2.0, 3.0)).to_enu();
        let point: nalgebra::Vector3<_> = ned.into();
        assert_eq!(point.x, 2.0);
        assert_eq!(point.y, 1.0);
        assert_eq!(point.z, -3.0);
    }
}
