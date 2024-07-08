use crate::{CoordinateFrameType, EastNorthUp, NorthEastDown};

/// A coordinate frame.
pub trait CoordinateFrame {
    /// The type of each coordinate value.
    type Type;

    /// The coordinate frame type.
    const COORDINATE_FRAME: CoordinateFrameType;

    /// Returns the coordinate frame of this instance.
    fn coordinate_frame(&self) -> CoordinateFrameType;

    /// Converts this type to a [`NorthEastDown`] instance.
    fn to_ned(&self) -> NorthEastDown<Self::Type>
    where
        Self::Type: Copy + SaturatingNeg<Output = Self::Type>;

    /// Converts this type to an [`EastNorthUp`] instance.
    fn to_enu(&self) -> EastNorthUp<Self::Type>
    where
        Self::Type: Copy + SaturatingNeg<Output = Self::Type>;

    /// Gets the value of the first dimension.
    fn x(&self) -> Self::Type
    where
        Self::Type: Clone;

    /// Gets the value of the second dimension.
    fn y(&self) -> Self::Type
    where
        Self::Type: Clone;

    /// Gets the value of the third dimension.
    fn z(&self) -> Self::Type
    where
        Self::Type: Clone;

    /// Gets the value of the first dimension.
    fn x_ref(&self) -> &Self::Type;

    /// Gets the value of the second dimension.
    fn y_ref(&self) -> &Self::Type;

    /// Gets the value of the third dimension.
    fn z_ref(&self) -> &Self::Type;

    /// Indicates whether this coordinate system is right-handed or left-handed.
    fn right_handed(&self) -> bool;
}

/// Marks a right-handed coordinate system.
pub trait RightHanded {}

/// Marks a left-handed coordinate system.
pub trait LeftHanded {}

/// Performs a saturating negation.
pub trait SaturatingNeg {
    type Output;

    /// Like [`Neg`](core::ops::Neg), but returns the maximum value on negation of
    /// a minimum value instead of overflowing.
    fn saturating_neg(self) -> Self::Output;
}

impl SaturatingNeg for i8 {
    type Output = Self;

    fn saturating_neg(self) -> Self {
        self.saturating_neg()
    }
}

impl SaturatingNeg for i16 {
    type Output = Self;

    fn saturating_neg(self) -> Self {
        self.saturating_neg()
    }
}
impl SaturatingNeg for i32 {
    type Output = Self;

    fn saturating_neg(self) -> Self {
        self.saturating_neg()
    }
}

impl SaturatingNeg for i64 {
    type Output = Self;

    fn saturating_neg(self) -> Self {
        self.saturating_neg()
    }
}

impl SaturatingNeg for i128 {
    type Output = Self;

    fn saturating_neg(self) -> Self {
        self.saturating_neg()
    }
}

impl SaturatingNeg for f32 {
    type Output = Self;

    fn saturating_neg(self) -> Self {
        -self
    }
}

impl SaturatingNeg for f64 {
    type Output = Self;

    fn saturating_neg(self) -> Self {
        -self
    }
}
