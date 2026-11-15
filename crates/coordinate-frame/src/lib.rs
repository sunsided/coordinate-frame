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
//! assert_eq!(axis, [0.0, 0.0, 1.0]);
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
    use crate::*;

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
        assert_eq!(axis, [0.0, 0.0, 1.0]);

        // map
        let ned2 = ned.map(|x| x * 2.0);
        assert_eq!(ned2.north(), 2.0);
        assert_eq!(ned2.east(), 4.0);
        assert_eq!(ned2.down(), -6.0);
    }

    #[test]
    fn map() {
        let ned = NorthEastDown::new(1.0, 2.0, -3.0);
        let ned2 = ned.map(|x| x * 2.0);
        assert_eq!(ned2.north(), 2.0);
        assert_eq!(ned2.east(), 4.0);
        assert_eq!(ned2.down(), -6.0);
    }

    #[test]
    fn construct() {
        let ned = NorthEastDown::new_from(CoordinateFrameType::SouthWestUp, 1.0, 2.0, 3.0)
            .expect("invalid conversion");
        assert_eq!(ned.north(), -1.0);
        assert_eq!(ned.east(), -2.0);
        assert_eq!(ned.down(), -3.0);
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

    /// Checks the invariants of a single frame type and returns its encoding.
    macro_rules! check_frame {
        ($frame:ident) => {{
            type F = $frame<f64>;
            let frame = F::COORDINATE_FRAME;

            // Base vectors are unit vectors in the frame's own coordinates.
            assert_eq!(F::x_axis(), [1.0, 0.0, 0.0], "{frame} x axis");
            assert_eq!(F::y_axis(), [0.0, 1.0, 0.0], "{frame} y axis");
            assert_eq!(F::z_axis(), [0.0, 0.0, 1.0], "{frame} z axis");

            // Round trip through NED is lossless and agrees with `construct_frame`.
            let value = F::new(1.0, 2.0, 3.0);
            let ned = NorthEastDown::from(value);
            assert_eq!(F::from(ned), value, "{frame} round trip");
            assert_eq!(
                NorthEastDown::new_from(frame, 1.0, 2.0, 3.0),
                Some(ned),
                "{frame} construct_frame"
            );

            // Handedness matches the physical orientation of the axes, measured in NED.
            let axis = |a: [f64; 3]| NorthEastDown::from(F::from_array(a));
            let orientation = axis(F::x_axis()).dot(&axis(F::y_axis()).cross(&axis(F::z_axis())));
            let expected = if value.right_handed() { 1.0 } else { -1.0 };
            assert_eq!(orientation, expected, "{frame} handedness");

            // The cross product describes the same physical vector in every frame.
            let a = NorthEastDown::new(1.0, 2.0, 3.0);
            let b = NorthEastDown::new(-2.0, 0.5, 4.0);
            let crossed = NorthEastDown::from(F::from(a).cross(&F::from(b)));
            assert_eq!(crossed, a.cross(&b), "{frame} cross product");

            u8::from(frame)
        }};
    }

    #[test]
    fn all_frames() {
        macro_rules! check_all {
            ($($frame:ident,)*) => { [$(check_frame!($frame)),*] };
        }
        let mut frames = check_all!(
            NorthEastDown,
            NorthEastUp,
            NorthWestDown,
            NorthWestUp,
            NorthDownEast,
            NorthDownWest,
            NorthUpEast,
            NorthUpWest,
            EastNorthDown,
            EastNorthUp,
            EastSouthDown,
            EastSouthUp,
            EastDownNorth,
            EastDownSouth,
            EastUpNorth,
            EastUpSouth,
            SouthEastDown,
            SouthEastUp,
            SouthWestDown,
            SouthWestUp,
            SouthDownEast,
            SouthDownWest,
            SouthUpEast,
            SouthUpWest,
            WestNorthDown,
            WestNorthUp,
            WestSouthDown,
            WestSouthUp,
            WestDownNorth,
            WestDownSouth,
            WestUpNorth,
            WestUpSouth,
            DownNorthEast,
            DownNorthWest,
            DownEastNorth,
            DownEastSouth,
            DownSouthEast,
            DownSouthWest,
            DownWestNorth,
            DownWestSouth,
            UpNorthEast,
            UpNorthWest,
            UpEastNorth,
            UpEastSouth,
            UpSouthEast,
            UpSouthWest,
            UpWestNorth,
            UpWestSouth,
        );
        frames.sort_unstable();
        for (expected, frame) in frames.into_iter().enumerate() {
            assert_eq!(
                usize::from(frame),
                expected,
                "every variant is covered exactly once"
            );
        }
    }

    #[test]
    fn try_from_slice() {
        let ned = NorthEastDown::try_from(&[1.0, 2.0, 3.0][..]).expect("length 3");
        assert_eq!(ned, &[1.0, 2.0, 3.0]);
        assert!(NorthEastDown::<f64>::try_from(&[1.0, 2.0][..]).is_err());
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
