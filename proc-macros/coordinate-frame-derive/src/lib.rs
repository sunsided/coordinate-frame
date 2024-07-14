//! Provides the `CoordinateFrame` derive macro.

use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, Data, DataEnum, DeriveInput, Fields, Ident, Lit};

const LATERAL: [&str; 2] = ["east", "west"];
const LONGITUDINAL: [&str; 2] = ["north", "south"];
const VERTICAL: [&str; 2] = ["down", "up"];
const MUTUALLY_EXCLUSIVE: [[&str; 2]; 3] = [LATERAL, LONGITUDINAL, VERTICAL];

#[proc_macro_derive(CoordinateFrame)]
pub fn derive_coordinate_frame(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    if let Data::Enum(data_enum) = input.data {
        process_enum(name, data_enum)
    } else {
        error_only_enums()
    }
}

/// Processes an enum of which we assume it is unit, i.e. (all) variants have no embedded values.
fn process_unit_enum(enum_name: Ident, data_enum: DataEnum) -> TokenStream {
    let mut parse_u8_arms = Vec::new();
    let mut defmt_arms = Vec::new();
    let mut display_arms = Vec::new();

    let impls = data_enum.variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        let variant_value = variant.discriminant.as_ref().map(|(_, expr)| {
            match expr {
                syn::Expr::Lit(syn::ExprLit { lit: Lit::Int(lit_int), .. }) => lit_int.base10_parse::<u8>().unwrap(),
                _ => panic!("Enum discriminant is not an integer literal"),
            }
        }).expect("Enum variants must have explicit u8 values");

        let variant_name_str = format!("{variant_name}");
        display_arms.push(quote! {
            #enum_name :: #variant_name  => f.write_str(#variant_name_str),
        });

        defmt_arms.push(quote! {
            #enum_name :: #variant_name  => defmt::write!(f, #variant_name_str),
        });

        parse_u8_arms.push(quote! {
            #variant_value => Ok(#enum_name :: #variant_name),
        });

        // Ignore the special "Other" variant.
        if variant_name == "Other" || variant_name == "Undefined" {
            quote! {}
        } else {
            let components = split_variant_name_into_components(&variant_name.to_string());

            // Implementations for each component.
            let mut components_impl = Vec::new();

            // Generate native accessors for the components.
            for (i, component) in components.iter().enumerate() {
                let component_name = format_ident!("{component}");
                let clone_function_name = format_ident!("{component}_clone");
                let with_function_name = format_ident!("with_{component}");
                let ref_function_name = format_ident!("{component}_ref");
                let mut_function_name = format_ident!("{component}_mut");
                let with_doc_str = format!("Consumes self and returns a new instance with the _{component}_ component set to the provided value.");
                let doc_str = format!("Returns the _{component}_ component of this coordinate.");
                let ref_doc_str = format!(
                    "Returns a reference to the _{component}_ component of this coordinate."
                );
                let mut_doc_str = format!(
                    "Returns a mutable reference to the _{component}_ component of this coordinate."
                );
                components_impl.push(quote! {
                    #[doc = #with_doc_str]
                    #[inline]
                    pub fn #with_function_name (mut self, #component_name: T) -> Self {
                        self.0[#i] = #component_name;
                        self
                    }

                    #[doc = #doc_str]
                    #[inline]
                    pub const fn #component_name (&self) -> T  where T: Copy {
                        self.0[#i]
                    }

                    #[inline]
                    fn #clone_function_name (&self) -> T  where T: Clone {
                        self.0[#i].clone()
                    }

                    #[doc = #ref_doc_str]
                    #[inline]
                    pub const fn #ref_function_name (&self) -> &T {
                        &self.0[#i]
                    }

                    #[doc = #mut_doc_str]
                    #[inline]
                    pub fn #mut_function_name (&mut self) -> &mut T {
                        &mut self.0[#i]
                    }
                });
            }

            // Generate derived pairs.
            let mut opposing_direction = Vec::new();
            for component  in components.iter() {
                let pair = MUTUALLY_EXCLUSIVE.iter().copied().find(|&pair| pair.contains(&component.as_str())).expect("Failed to identify component pair");
                let other = pair.iter().copied().find(|&other| !other.eq(component.as_str())).expect("Failed to find component's opposite direction");
                opposing_direction.push(other);

                let component_name = format_ident!("{component}");
                let clone_component_name = format_ident!("{component}_clone");
                let other_name = format_ident!("{other}");
                let clone_other_name = format_ident!("{other}_clone");
                let doc_str = format!("Returns the _{other}_ component of this coordinate. This component is not a native axis of the coordinate frame and is derived from the [`{component}`](Self::{component}) component at runtime.");

                components_impl.push(quote! {
                    #[doc = #doc_str]
                    #[inline]
                    pub fn #other_name (&self) -> T  where T: Copy + SaturatingNeg<Output = T> {
                        let component = self . #component_name();
                        component.saturating_neg()
                    }

                    #[inline]
                    fn #clone_other_name (&self) -> T  where T: Clone + SaturatingNeg<Output = T> {
                        let component = self . #clone_component_name();
                        component.saturating_neg()
                    }
                });
            }

            // Create flipped version.
            let flipped_name = String::from_iter(opposing_direction.iter().map(|component| capitalize(component)));
            let flipped_ident = format_ident!("{}", flipped_name);
            let flip_doc = format!("Flips this coordinate frame into its opposite frame, [`{flipped_name}`]");
            components_impl.push(quote! {
                #[doc = #flip_doc]
                #[inline]
                pub fn flip_frame(&self) -> #flipped_ident <T>
                where
                    T: Copy + SaturatingNeg<Output = T>
                {
                    (*self).into()
                }
            });

            // Create constructor.
            let first_component = format_ident!("{}", &components[0]);
            let second_component = format_ident!("{}", &components[1]);
            let third_component = format_ident!("{}", &components[2]);
            let new_doc = format!("Creates a new [`{variant_name}`] instance from its _{}_, _{}_ and _{}_ components.",
                &components[0], &components[1], &components[2]
            );

            // Provide conversion to North, East, Down
            let north = String::from("north");
            let east = String::from("east");
            let down = String::from("down");
            if variant_name != "NorthEastDown" && components.contains(&north) && components.contains(&east) && components.contains(&down) {
                components_impl.push(quote! {
                    /// Converts this type to a [`NorthEastDown`] instance.
                    pub const fn to_ned(&self) -> NorthEastDown<T> where T: Copy {
                        let north = self.north();
                        let east = self.east();
                        let down = self.down();
                        NorthEastDown::new(north, east, down)
                    }
                });
            } else {
                components_impl.push(quote! {
                    /// Converts this type to a [`NorthEastDown`] instance.
                    pub fn to_ned(&self) -> NorthEastDown<T> where T: Copy + SaturatingNeg<Output = T> {
                        let north = self.north();
                        let east = self.east();
                        let down = self.down();
                        NorthEastDown::new(north, east, down)
                    }
                });
            }

            // Provide conversion to East, North, Up
            let up = String::from("up");
            if variant_name != "EastNorthUp" && components.contains(&east) && components.contains(&north) && components.contains(&up) {
                components_impl.push(quote! {
                    /// Converts this type to an [`NorthEastDown`] instance.
                    pub const fn to_enu(&self) -> EastNorthUp<T> where T: Copy {
                        let east = self.east();
                        let north = self.north();
                        let up = self.up();
                        EastNorthUp::new(east, north, up)
                    }
                });
            } else {
                components_impl.push(quote! {
                    /// Converts this type to an [`EastNorthUp`] instance.
                    pub fn to_enu(&self) -> EastNorthUp<T> where T: Copy + SaturatingNeg<Output = T> {
                        let east = self.east();
                        let north = self.north();
                        let up = self.up();
                        EastNorthUp::new(east, north, up)
                    }
                });
            }

            // Type conversion implementations.
            let mut conversion_impl = Vec::new();
            for other_variant in data_enum.variants.iter().filter(|other| other.ident != *variant_name) {
                // Skip the generic fallback.
                let other_variant = &other_variant.ident;
                if other_variant == "Other" || other_variant == "Undefined" {
                    continue;
                }

                let components = split_variant_name_into_components(&other_variant.to_string());
                let first_component = format_ident!("{}", &components[0]);
                let second_component = format_ident!("{}", &components[1]);
                let third_component = format_ident!("{}", &components[2]);

                let clone_first_component = format_ident!("{}_clone", &components[0]);
                let clone_second_component = format_ident!("{}_clone", &components[1]);
                let clone_third_component = format_ident!("{}_clone", &components[2]);

                conversion_impl.push(quote! {
                    impl<T> From<#variant_name <T>> for #other_variant <T> where T: Clone + SaturatingNeg<Output = T> {
                        fn from(value: #variant_name <T>) -> #other_variant <T> {
                            let #first_component = value. #clone_first_component ();
                            let #second_component = value. #clone_second_component ();
                            let #third_component = value. #clone_third_component ();
                            #other_variant :: new(#first_component, #second_component, #third_component)
                        }
                    }
                });
            }

            // Handedness
            let right_handed = is_right_handed(&components[0], &components[1], &components[2]);

            let mut handedness_impl = Vec::new();
            if right_handed {
                handedness_impl.push(quote!{
                    impl<T> RightHanded for #variant_name <T> {}
                });
            } else {
                handedness_impl.push(quote!{
                    impl<T> LeftHanded for #variant_name <T> {}
                });
            }

            // Base vectors
            // TODO: Remove, ambiguous
            let x_axis_vec = axis_def_t(&components[0]);
            let y_axis_vec = axis_def_t(&components[1]);
            let z_axis_vec = axis_def_t(&components[2]);

            // Documentation for x, y and z.
            let x_doc = format!("For this type, this represents the [`{first_component}`](Self::{first_component}) direction.");
            let y_doc = format!("For this type, this represents the [`{second_component}`](Self::{second_component}) direction.");
            let z_doc = format!("For this type, this represents the [`{third_component}`](Self::{third_component}) direction.");

            // Long documentation for the type.
            let handedness = if right_handed {
                "right-handed"
            }  else { "left-handed" };

            let handedness_emoji = if right_handed {
                "🫱"
            }  else { "🫲" };

            let mut doc_long = format!("# A {}, {} and {} frame", components[0], components[1], components[2]);
            if variant_name == "NorthEastDown" {
                doc_long.push_str(&format!(" ({handedness}, aeronautics)"));
            } else if variant_name == "EastNorthUp" {
                doc_long.push_str(&format!(" ({handedness}, geography)"));
            } else {
                doc_long.push_str(&format!(" ({handedness})"));
            }

            let x_dir_human = axis_direction_human(&components[0]);
            let y_dir_human = axis_direction_human(&components[1]);
            let z_dir_human = axis_direction_human(&components[2]);

            let doc_long_second = format!("This resembles a {handedness_emoji} {} coordinate system representing the {}, {} and {} directions, respectively.\n\nIt is encoded by [`CoordinateFrameType::{variant_name}`](CoordinateFrameType::{variant_name}).",
                                          handedness,
                                          x_dir_human,
                                          y_dir_human,
                                          z_dir_human);

            let x_doc_long = format!("* [`x`](Self::x) represents [`{}`](Self::{}), i.e. the {} axis with positive values representing \"{}\".",
                                     components[0],
                                     components[0],
                                     axis_direction(&components[0]),
                                     x_dir_human);
            let y_doc_long = format!("* [`y`](Self::y) represents [`{}`](Self::{}), i.e. the {} axis with positive values representing \"{}\".",
                                     components[1],
                                     components[1],
                                     axis_direction(&components[1]),
                                     y_dir_human);
            let z_doc_long = format!("* [`z`](Self::z) represents [`{}`](Self::{}), i.e. the {} axis with positive values representing \"{}\".",
                                     components[2],
                                     components[2],
                                     axis_direction(&components[2]),
                                     z_dir_human);

            // ASCII art
            let (up_down, up_down_axis) = if VERTICAL.contains(&components[0].as_str()) {
                (&components[0], "x")
            } else if VERTICAL.contains(&components[1].as_str()) {
                (&components[1], "y")
            } else {
                (&components[2], "z")
            };
            let (north_south, north_south_axis) = if LONGITUDINAL.contains(&components[0].as_str()) {
                (&components[0], "x")
            } else if LONGITUDINAL.contains(&components[1].as_str()) {
                (&components[1], "y")
            } else {
                (&components[2], "z")
            };
            let (east_west, east_west_axis) = if LATERAL.contains(&components[0].as_str()) {
                (&components[0], "x")
            } else if LATERAL.contains(&components[1].as_str()) {
                (&components[1], "y")
            } else {
                (&components[2], "z")
            };

            let ascii = ascii_art(up_down, north_south, east_west, up_down_axis, north_south_axis, east_west_axis);
            let ascii_art_doc = format!(r#"```plain
{ascii}
```"#);

            quote! {
                #[doc = #doc_long]
                #[doc = #doc_long_second]
                /// ## Axis descriptions
                #[doc = #x_doc_long]
                #[doc = #y_doc_long]
                #[doc = #z_doc_long]
                #[doc = #ascii_art_doc]
                #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
                #[repr(C)]
                pub struct #variant_name <T>([T; 3]);

                impl<T> core::fmt::Display for #variant_name <T> where T: core::fmt::Display {
                    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                        use core::fmt::Write;
                        f.write_str(#variant_name_str)?;
                        f.write_char('(')?;
                        core::fmt::Display::fmt(&self.0[0], f)?;
                        f.write_str(", ")?;
                        core::fmt::Display::fmt(&self.0[1], f)?;
                        f.write_str(", ")?;
                        core::fmt::Display::fmt(&self.0[2], f)?;
                        f.write_char(')')
                    }
                }

                #[cfg(feature = "defmt")]
                #[cfg_attr(docsrs, doc(cfg(feature = "defmt")))]
                impl<T> defmt::Format for #variant_name <T> where T: defmt::Format {
                    fn format(&self, f: defmt::Formatter) {
                        defmt::write!(f, "{}({}, {}, {})", #variant_name_str, self.0[0], self.0[1], self.0[2])
                    }
                }

                impl<T> #variant_name <T> {
                    /// The coordinate frame type.
                    pub const COORDINATE_FRAME: #enum_name = #enum_name :: #variant_name;

                    #[doc = #new_doc]
                    pub const fn new(#first_component: T, #second_component: T, #third_component: T) -> Self {
                        Self([#first_component, #second_component, #third_component])
                    }

                    /// Constructs an instance from an array.
                    ///
                    /// Be mindful not to directly pass a different coordinate frame into
                    /// this function unless you want to strictly re-interpret the values.
                    pub const fn from_array(vec: [T; 3]) -> Self {
                        Self(vec)
                    }

                    /// Constructs an instance from a slice.
                    ///
                    /// Be mindful not to directly pass a different coordinate frame into
                    /// this function unless you want to strictly re-interpret the values.
                    pub fn from_slice(vec: &[T]) -> Self
                    where
                        T: Clone
                    {
                        assert_eq!(vec.len(), 3, "The provided slice must have length 3");
                        let z = vec[2].clone();
                        let y = vec[1].clone();
                        let x = vec[0].clone();
                        Self([x, y, z])
                    }

                    /// Gets the value of the first dimension.
                    #[doc = #x_doc]
                    pub fn x(&self) -> T where T: Clone {
                        self.0[0].clone()
                    }

                    /// Gets the value of the second dimension.
                    #[doc = #y_doc]
                    pub fn y(&self) -> T where T: Clone {
                        self.0[1].clone()
                    }

                    /// Gets the value of the third dimension.
                    #[doc = #z_doc]
                    pub fn z(&self) -> T where T: Clone {
                        self.0[2].clone()
                    }

                    /// Gets a reference to the value of the first dimension.
                    #[doc = #x_doc]
                    pub fn x_ref(&self) -> &T {
                        &self.0[0]
                    }

                    /// Gets a reference to the value of the second dimension.
                    #[doc = #y_doc]
                    pub fn y_ref(&self) -> &T {
                        &self.0[1]
                    }

                    /// Gets a reference to the value of the third dimension.
                    #[doc = #z_doc]
                    pub fn z_ref(&self) -> &T {
                        &self.0[2]
                    }

                    /// Gets a mutable reference to the value of the first dimension.
                    #[doc = #x_doc]
                    pub fn x_mut(&mut self) -> &mut T {
                        &mut self.0[0]
                    }

                    /// Gets a mutable reference to the value of the second dimension.
                    #[doc = #y_doc]
                    pub fn y_mut(&mut self) -> &mut T {
                        &mut self.0[1]
                    }

                    /// Gets a mutable reference to the value of the third dimension.
                    #[doc = #z_doc]
                    pub fn z_mut(&mut self) -> &mut T {
                        &mut self.0[2]
                    }

                    /// Consumes self and returns its inner value.
                    pub const fn into_inner(self) -> [T; 3] where T: Copy {
                        self.0
                    }

                    /// Returns the coordinate frame of this instance.
                    ///
                    /// This is the same as [`COORDINATE_FRAME`](Self::COORDINATE_FRAME), except
                    /// that it is available as a function on an instance.
                    pub const fn coordinate_frame(&self) -> #enum_name {
                        Self::COORDINATE_FRAME
                    }

                    /// Indicates whether this coordinate system is right-handed or left-handed.
                    pub const fn right_handed(&self) -> bool {
                        #right_handed
                    }

                    /// Returns the base vector for the `x` axis in the global frame.
                    #[deprecated(since = "0.4.0", note="This method is ambiguous regarding its reference frame.")]
                    pub fn x_axis() -> [T; 3] where T: ZeroOne<Output = T> + core::ops::Neg<Output = T> {
                        #x_axis_vec
                    }

                    /// Returns the base vector for the `y` axis in the global frame.
                    #[deprecated(since = "0.4.0", note="This method is ambiguous regarding its reference frame.")]
                    pub fn y_axis() -> [T; 3] where T: ZeroOne<Output = T> + core::ops::Neg<Output = T> {
                        #y_axis_vec
                    }

                    /// Returns the base vector for the `z` axis in the global frame.
                    #[deprecated(since = "0.4.0", note="This method is ambiguous regarding its reference frame.")]
                    pub fn z_axis() -> [T; 3] where T: ZeroOne<Output = T> + core::ops::Neg<Output = T> {
                        #z_axis_vec
                    }

                    /// Calculates the squared norm of the components.
                    ///
                    /// ## Panics
                    /// This operation may overflow.
                    pub fn norm_sq(&self) -> T where T: Clone + core::ops::Mul<T, Output = T> + core::ops::Add<T, Output = T> {
                        let x = self.x().clone();
                        let y = self.y().clone();
                        let z = self.z().clone();
                        x.clone() * x + y.clone() * y + z.clone() * z
                    }

                    /// Calculates the cross product (outer product) of two coordinates.
                    ///
                    /// ## Panics
                    /// This operation may overflow.
                    pub fn cross(&self, rhs: &Self) -> Self where T: Clone + core::ops::Mul<T, Output = T> + core::ops::Sub<T, Output = T> {
                        Self([
                            self[1].clone() * rhs[2].clone() - self[2].clone() * rhs[1].clone(),
                            self[2].clone() * rhs[0].clone() - self[0].clone() * rhs[2].clone(),
                            self[0].clone() * rhs[1].clone() - self[1].clone() * rhs[0].clone()
                        ])
                    }

                    /// Calculates the dot product (inner product) of two coordinates.
                    ///
                    /// ## Panics
                    /// This operation may overflow.
                    pub fn dot(&self, rhs: &Self) -> T where T: Clone + core::ops::Mul<T, Output = T> + core::ops::Add<T, Output = T> {
                        self[0].clone() * rhs[0].clone() + self[1].clone() * rhs[1].clone() + self[2].clone() * rhs[2].clone()
                    }

                    #(#components_impl)*
                }

                impl<T> CoordinateFrame for #variant_name <T> {
                    type Type = T;

                    /// The coordinate frame.
                    const COORDINATE_FRAME: #enum_name = #enum_name :: #variant_name;

                    /// Returns the coordinate frame of this instance.
                    fn coordinate_frame(&self) -> #enum_name {
                        Self::COORDINATE_FRAME
                    }

                    /// Converts this type to a [`NorthEastDown`] instance.
                    fn to_ned(&self) -> NorthEastDown<Self::Type>
                    where
                        Self::Type: Copy + SaturatingNeg<Output = Self::Type> {
                        self.to_ned()
                    }

                    /// Converts this type to an [`EastNorthUp`] instance.
                    fn to_enu(&self) -> EastNorthUp<Self::Type>
                    where
                        Self::Type: Copy + SaturatingNeg<Output = Self::Type> {
                        self.to_enu()
                    }

                    /// Gets the value of the first dimension.
                    #[doc = #x_doc]
                    fn x(&self) -> Self::Type where Self::Type: Clone {
                        self.x()
                    }

                    /// Gets the value of the second dimension.
                    #[doc = #y_doc]
                    fn y(&self) -> Self::Type where Self::Type: Clone {
                        self.y()
                    }

                    /// Gets the value of the third dimension.
                    #[doc = #z_doc]
                    fn z(&self) -> Self::Type where Self::Type: Clone {
                        self.z()
                    }

                    /// Gets a reference to the value of the first dimension.
                    #[doc = #x_doc]
                    fn x_ref(&self) -> &Self::Type {
                        self.x_ref()
                    }

                    /// Gets a reference to the value of the second dimension.
                    #[doc = #y_doc]
                    fn y_ref(&self) -> &Self::Type {
                        self.y_ref()
                    }

                    /// Gets a reference to the value of the third dimension.
                    #[doc = #z_doc]
                    fn z_ref(&self) -> &Self::Type {
                        self.z_ref()
                    }

                    /// Gets a mutable reference to the value of the first dimension.
                    #[doc = #x_doc]
                    fn x_mut(&mut self) -> &mut Self::Type {
                        self.x_mut()
                    }

                    /// Gets a mutable reference to the value of the second dimension.
                    #[doc = #y_doc]
                    fn y_mut(&mut self) -> &mut Self::Type {
                        self.y_mut()
                    }

                    /// Gets a mutable reference to the value of the third dimension.
                    #[doc = #z_doc]
                    fn z_mut(&mut self) -> &mut Self::Type {
                        self.z_mut()
                    }

                    /// Indicates whether this coordinate system is right-handed or left-handed.
                    fn right_handed(&self) -> bool {
                        self.right_handed()
                    }

                    /// Returns the base vector for the `x` axis.
                    #[inline]
                    #[must_use]
                    fn x_axis() -> [Self::Type; 3] where Self::Type: ZeroOne<Output = Self::Type> + core::ops::Neg<Output = Self::Type> {
                        Self::x_axis()
                    }

                    /// Returns the base vector for the `y` axis.
                    #[inline]
                    #[must_use]
                    fn y_axis() -> [Self::Type; 3] where Self::Type: ZeroOne<Output = Self::Type> + core::ops::Neg<Output = Self::Type> {
                        Self::y_axis()
                    }

                    /// Returns the base vector for the `z` axis.
                    #[inline]
                    #[must_use]
                    fn z_axis() -> [Self::Type; 3] where Self::Type: ZeroOne<Output = Self::Type> + core::ops::Neg<Output = Self::Type> {
                        Self::z_axis()
                    }
                }

                impl<T> From<#variant_name <T>> for [T; 3] {
                    fn from(value: #variant_name <T>) -> [T; 3] {
                        value.0
                    }
                }

                impl<T> From<#variant_name <T>> for (T, T, T) {
                    fn from(value: #variant_name <T>) -> (T, T, T) {
                        let [x, y, z] = value.0;
                        (x, y, z)
                    }
                }

                impl<T> From<[T; 3]> for #variant_name <T> {
                    fn from(value: [T; 3]) -> #variant_name <T> {
                        #variant_name (value)
                    }
                }

                #[cfg(feature = "micromath")]
                #[cfg_attr(docsrs, doc(cfg(feature = "micromath")))]
                impl From<micromath::vector::F32x3> for #variant_name <f32> {
                    fn from(value: micromath::vector::F32x3) -> #variant_name <f32> {
                        Self([value.x, value.y, value.z])
                    }
                }

                #[cfg(feature = "micromath")]
                #[cfg_attr(docsrs, doc(cfg(feature = "micromath")))]
                impl From<#variant_name <f32>> for micromath::vector::F32x3 {
                    fn from(value: #variant_name <f32>) -> micromath::vector::F32x3 {
                        let [x, y, z] = value.0;
                        Self { x, y, z }
                    }
                }

                #[cfg(feature = "micromath")]
                #[cfg_attr(docsrs, doc(cfg(feature = "micromath")))]
                impl From<micromath::vector::U32x3> for #variant_name <u32> {
                    fn from(value: micromath::vector::U32x3) -> #variant_name <u32> {
                        Self([value.x, value.y, value.z])
                    }
                }

                #[cfg(feature = "micromath")]
                #[cfg_attr(docsrs, doc(cfg(feature = "micromath")))]
                impl From<#variant_name <u32>> for micromath::vector::U32x3 {
                    fn from(value: #variant_name <u32>) -> micromath::vector::U32x3 {
                        let [x, y, z] = value.0;
                        Self { x, y, z }
                    }
                }

                #[cfg(feature = "micromath")]
                #[cfg_attr(docsrs, doc(cfg(feature = "micromath")))]
                impl From<micromath::vector::I32x3> for #variant_name <i32> {
                    fn from(value: micromath::vector::I32x3) -> #variant_name <i32> {
                        Self([value.x, value.y, value.z])
                    }
                }

                #[cfg(feature = "micromath")]
                #[cfg_attr(docsrs, doc(cfg(feature = "micromath")))]
                impl From<#variant_name <i32>> for micromath::vector::I32x3 {
                    fn from(value: #variant_name <i32>) -> micromath::vector::I32x3 {
                        let [x, y, z] = value.0;
                        Self { x, y, z }
                    }
                }

                #[cfg(feature = "micromath")]
                #[cfg_attr(docsrs, doc(cfg(feature = "micromath")))]
                impl From<micromath::vector::U16x3> for #variant_name <u16> {
                    fn from(value: micromath::vector::U16x3) -> #variant_name <u16> {
                        Self([value.x, value.y, value.z])
                    }
                }

                #[cfg(feature = "micromath")]
                #[cfg_attr(docsrs, doc(cfg(feature = "micromath")))]
                impl From<#variant_name <u16>> for micromath::vector::U16x3 {
                    fn from(value: #variant_name <u16>) -> micromath::vector::U16x3 {
                        let [x, y, z] = value.0;
                        Self { x, y, z }
                    }
                }

                #[cfg(feature = "micromath")]
                #[cfg_attr(docsrs, doc(cfg(feature = "micromath")))]
                impl From<micromath::vector::I16x3> for #variant_name <i16> {
                    fn from(value: micromath::vector::I16x3) -> #variant_name <i16> {
                        Self([value.x, value.y, value.z])
                    }
                }

                #[cfg(feature = "micromath")]
                #[cfg_attr(docsrs, doc(cfg(feature = "micromath")))]
                impl From<#variant_name <i16>> for micromath::vector::I16x3 {
                    fn from(value: #variant_name <i16>) -> micromath::vector::I16x3 {
                        let [x, y, z] = value.0;
                        Self { x, y, z }
                    }
                }

                #[cfg(feature = "micromath")]
                #[cfg_attr(docsrs, doc(cfg(feature = "micromath")))]
                impl From<micromath::vector::U8x3> for #variant_name <u8> {
                    fn from(value: micromath::vector::U8x3) -> #variant_name <u8> {
                        Self([value.x, value.y, value.z])
                    }
                }

                #[cfg(feature = "micromath")]
                #[cfg_attr(docsrs, doc(cfg(feature = "micromath")))]
                impl From<#variant_name <u8>> for micromath::vector::U8x3 {
                    fn from(value: #variant_name <u8>) -> micromath::vector::U8x3 {
                        let [x, y, z] = value.0;
                        Self { x, y, z }
                    }
                }

                #[cfg(feature = "micromath")]
                #[cfg_attr(docsrs, doc(cfg(feature = "micromath")))]
                impl From<micromath::vector::I8x3> for #variant_name <i8> {
                    fn from(value: micromath::vector::I8x3) -> #variant_name <i8> {
                        Self([value.x, value.y, value.z])
                    }
                }

                #[cfg(feature = "micromath")]
                #[cfg_attr(docsrs, doc(cfg(feature = "micromath")))]
                impl From<#variant_name <i8>> for micromath::vector::I8x3 {
                    fn from(value: #variant_name <i8>) -> micromath::vector::I8x3 {
                        let [x, y, z] = value.0;
                        Self { x, y, z }
                    }
                }

                impl<T> core::convert::AsRef<[T; 3]> for #variant_name <T> {
                    fn as_ref(&self) -> &[T; 3] {
                        &self.0
                    }
                }

                impl<T> core::convert::AsRef<[T]> for #variant_name <T> {
                    fn as_ref(&self) -> &[T] {
                        &self.0
                    }
                }

                impl<T> core::convert::AsMut<[T; 3]> for #variant_name <T> {
                    fn as_mut(&mut self) -> &mut [T; 3] {
                        &mut self.0
                    }
                }

                impl<T> core::convert::AsMut<[T]> for #variant_name <T> {
                    fn as_mut(&mut self) -> &mut [T] {
                        &mut self.0
                    }
                }

                impl<T> core::ops::Deref for #variant_name <T> {
                    type Target = [T; 3];

                    fn deref(&self) -> &Self::Target {
                        &self.0
                    }
                }

                impl<T> core::ops::DerefMut for #variant_name <T> {
                    fn deref_mut(&mut self) -> &mut Self::Target {
                        &mut self.0
                    }
                }

                impl<T> core::cmp::PartialEq<&[T; 3]> for #variant_name <T> where T: core::cmp::PartialEq<T> {
                    fn eq(&self, other: &&[T; 3]) -> bool {
                        self.0.eq(*other)
                    }
                }

                #(#handedness_impl)*
                #(#conversion_impl)*

                #[cfg(feature = "nalgebra")]
                #[cfg_attr(docsrs, doc(cfg(feature = "nalgebra")))]
                impl<T> core::convert::From<nalgebra::Point3<T>> for #variant_name <T>
                where
                    T: nalgebra::Scalar + Copy
                {
                    fn from(value: nalgebra::Point3<T>) -> #variant_name <T> {
                        Self::new(value.x, value.y, value.z)
                    }
                }

                #[cfg(feature = "nalgebra")]
                #[cfg_attr(docsrs, doc(cfg(feature = "nalgebra")))]
                impl<T> core::convert::From<#variant_name <T>> for nalgebra::Point3<T>
                where
                    T: nalgebra::Scalar
                {
                    fn from(value: #variant_name <T>) -> nalgebra::Point3<T> {
                        let [x, y, z] = value.0;
                        Self::new(x, y, z)
                    }
                }

                #[cfg(feature = "nalgebra")]
                #[cfg_attr(docsrs, doc(cfg(feature = "nalgebra")))]
                impl<T> core::convert::From<nalgebra::Vector3<T>> for #variant_name <T>
                where
                    T: nalgebra::Scalar + Copy
                {
                    fn from(value: nalgebra::Vector3<T>) -> #variant_name <T> {
                        Self::new(value.x, value.y, value.z)
                    }
                }

                #[cfg(feature = "nalgebra")]
                #[cfg_attr(docsrs, doc(cfg(feature = "nalgebra")))]
                impl<T> core::convert::From<#variant_name <T>> for nalgebra::Vector3<T>
                where
                    T: nalgebra::Scalar
                {
                    fn from(value: #variant_name <T>) -> nalgebra::Vector3<T> {
                        let [x, y, z] = value.0;
                        Self::new(x, y, z)
                    }
                }

                impl<T> core::ops::Add<T> for #variant_name <T>
                where
                    T: core::ops::Add<T, Output = T> + Clone
                {
                    type Output = #variant_name <T>;

                    fn add(self, rhs: T) -> Self::Output {
                        let [x, y, z] = self.0;
                        Self::new(x + rhs.clone(), y + rhs.clone(), z + rhs)
                    }
                }

                impl<T> core::ops::AddAssign<T> for #variant_name <T>
                where
                    T: core::ops::AddAssign<T> + Clone
                {
                    fn add_assign(&mut self, rhs: T) {
                        self.0[0] += rhs.clone();
                        self.0[1] += rhs.clone();
                        self.0[2] += rhs;
                    }
                }

                impl<T> core::ops::Add<#variant_name <T>> for #variant_name <T>
                where
                    T: core::ops::Add<T, Output = T> + Clone
                {
                    type Output = #variant_name <T>;

                    fn add(self, rhs: #variant_name <T>) -> Self::Output {
                        let [x, y, z] = self.0;
                        let [x2, y2, z2] = rhs.0;
                        Self::new(x + x2, y + y2, z + z2)
                    }
                }

                impl<T> core::ops::Sub<T> for #variant_name <T>
                where
                    T: core::ops::Sub<T, Output = T> + Clone
                {
                    type Output = #variant_name <T>;

                    fn sub(self, rhs: T) -> Self::Output {
                        let [x, y, z] = self.0;
                        Self::new(x - rhs.clone(), y - rhs.clone(), z - rhs)
                    }
                }

                impl<T> core::ops::SubAssign<T> for #variant_name <T>
                where
                    T: core::ops::SubAssign<T> + Clone
                {
                    fn sub_assign(&mut self, rhs: T) {
                        self.0[0] -= rhs.clone();
                        self.0[1] -= rhs.clone();
                        self.0[2] -= rhs;
                    }
                }

                impl<T> core::ops::Sub<#variant_name <T>> for #variant_name <T>
                where
                    T: core::ops::Sub<T, Output = T> + Clone
                {
                    type Output = #variant_name <T>;

                    fn sub(self, rhs: #variant_name <T>) -> Self::Output {
                        let [x, y, z] = self.0;
                        let [x2, y2, z2] = rhs.0;
                        Self::new(x - x2, y - y2, z - z2)
                    }
                }

                impl<T> core::ops::Mul<T> for #variant_name <T>
                where
                    T: core::ops::Mul<T, Output = T> + Clone
                {
                    type Output = #variant_name <T>;

                    fn mul(self, rhs: T) -> Self::Output {
                        let [x, y, z] = self.0;
                        Self::new(x * rhs.clone(), y * rhs.clone(), z * rhs)
                    }
                }

                impl<T> core::ops::MulAssign<T> for #variant_name <T>
                where
                    T: core::ops::MulAssign<T> + Clone
                {
                    fn mul_assign(&mut self, rhs: T) {
                        self.0[0] *= rhs.clone();
                        self.0[1] *= rhs.clone();
                        self.0[2] *= rhs;
                    }
                }

                impl<T> core::ops::Div<T> for #variant_name <T>
                where
                    T: core::ops::Div<T, Output = T> + Clone
                {
                    type Output = #variant_name <T>;

                    fn div(self, rhs: T) -> Self::Output {
                        let [x, y, z] = self.0;
                        Self::new(x / rhs.clone(), y / rhs.clone(), z / rhs)
                    }
                }

                impl<T> core::ops::DivAssign<T> for #variant_name <T>
                where
                    T: core::ops::DivAssign<T> + Clone
                {
                    fn div_assign(&mut self, rhs: T) {
                        self.0[0] /= rhs.clone();
                        self.0[1] /= rhs.clone();
                        self.0[2] /= rhs;
                    }
                }
            }
        }
    });

    let expanded = quote! {
        #(#impls)*

        impl From<#enum_name> for u8 {
            fn from(value: #enum_name) -> u8 {
                value as u8
            }
        }

        impl From<&#enum_name> for u8 {
            fn from(value: &#enum_name) -> u8 {
                *value as u8
            }
        }

        impl core::convert::TryFrom<u8> for #enum_name {
            type Error = ParseCoordinateFrameError;

            fn try_from(value: u8) -> Result<#enum_name, Self::Error> {
                match value {
                    #(#parse_u8_arms)*
                    _ => Err(ParseCoordinateFrameError::UnknownVariant)
                }
            }
        }

        impl core::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    #(#display_arms)*
                }
            }
        }

        #[cfg(feature = "defmt")]
        #[cfg_attr(docsrs, doc(cfg(feature = "defmt")))]
        impl defmt::Format for #enum_name {
            fn format(&self, f: defmt::Formatter) {
                match self {
                    #(#defmt_arms)*
                }
            }
        }
    };
    TokenStream::from(expanded)
}

/// Processes an enum and returns an error if it is not unit.
fn process_enum(name: Ident, data_enum: DataEnum) -> TokenStream {
    let is_unit = data_enum
        .variants
        .iter()
        .all(|variant| matches!(variant.fields, Fields::Unit));
    if !is_unit {
        // Emit a compile-time error if any variant is non-trivial
        let error_message = format!(
            "The enum `{}` must have unit variants only to derive CoordinateFrame.",
            name
        );
        let expanded = quote! {
            compile_error!(#error_message);
        };
        return TokenStream::from(expanded);
    }

    process_unit_enum(name, data_enum)
}

/// Returns a compile-time error indicating that only `enum` types can derive `CoordinateFrame`.
fn error_only_enums() -> TokenStream {
    let error_message = "`CoordinateFrame` can only be derived for enums.".to_string();
    let expanded = quote! {
        compile_error!(#error_message);
    };
    TokenStream::from(expanded)
}

/// Splits an UpperCamelCase string into components
fn split_variant_name_into_components(input: &str) -> [String; 3] {
    let mut components = Vec::new();
    // Find an upper-case index, then slice the string until there
    // and push it into the components vector. Update the slice start accordingly.
    let mut start = 0;
    for (i, c) in input.char_indices() {
        if c.is_uppercase() && i != 0 {
            components.push(input[start..i].to_lowercase());
            start = i;
        }
    }
    components.push(input[start..].to_lowercase());
    components
        .try_into()
        .expect("Expected exactly three components")
}

fn axis_direction(axis: &str) -> &str {
    match axis {
        "east" => "lateral",
        "west" => "lateral",
        "north" => "longitudinal",
        "south" => "longitudinal",
        "up" => "vertical",
        "down" => "vertical",
        _ => unreachable!(),
    }
}

fn axis_direction_human(axis: &str) -> &str {
    match axis {
        "east" => "right",
        "west" => "left",
        "north" => "forward",
        "south" => "backward",
        "up" => "up",
        "down" => "down",
        _ => unreachable!(),
    }
}

fn is_right_handed(first: &str, second: &str, third: &str) -> bool {
    let first = axis_vec(first);
    let second = axis_vec(second);
    let third = axis_vec(third);

    let cross = cross(first, second);
    vectors_equal(cross, third)
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn vectors_equal(v1: [f32; 3], v2: [f32; 3]) -> bool {
    const EPSILON: f32 = 1e-6;
    (v1[0] - v2[0]).abs() < EPSILON
        && (v1[1] - v2[1]).abs() < EPSILON
        && (v1[2] - v2[2]).abs() < EPSILON
}

fn axis_vec(axis: &str) -> [f32; 3] {
    match axis {
        "north" => [0.0, 1.0, 0.0],
        "south" => [0.0, -1.0, 0.0],
        "east" => [1.0, 0.0, 0.0],
        "west" => [-1.0, 0.0, 0.0],
        "up" => [0.0, 0.0, 1.0],
        "down" => [0.0, 0.0, -1.0],
        _ => unreachable!(),
    }
}

fn axis_def_t(axis: &str) -> impl ToTokens {
    match axis {
        "north" => quote! { [T::zero(), T::one(), T::zero()] },
        "south" => quote! { [T::zero(), T::one().neg(), T::zero()] },
        "east" => quote! { [T::one(), T::zero(), T::zero()] },
        "west" => quote! { [T::one().neg(), T::one(), T::zero()] },
        "up" => quote! { [T::zero(), T::zero(), T::one()] },
        "down" => quote! { [T::zero(), T::zero(), T::one().neg()] },
        _ => unreachable!(),
    }
}

fn capitalize(axis: &str) -> &str {
    match axis {
        "north" => "North",
        "south" => "South",
        "east" => "East",
        "west" => "West",
        "up" => "Up",
        "down" => "Down",
        _ => unreachable!(),
    }
}

fn up_west_south(up: &str, south: &str, west: &str) -> String {
    format!(
        r#"           {up} (up)
           |
           |
           |
   {west} ______|
(west)    /
         /
        /
       {south} (south)
"#
    )
}

fn up_east_south(up: &str, south: &str, east: &str) -> String {
    format!(
        r#"    {up} (up)
    |
    |
    |
    |______ {east} (east)
   /
  /
 /
{south} (south)
"#
    )
}

fn up_east_north(up: &str, north: &str, east: &str) -> String {
    format!(
        r#"{up} (up)
|   {north} (north)
|  /
| /
|/______ {east} (east)
"#
    )
}

fn up_west_north(up: &str, north: &str, west: &str) -> String {
    format!(
        r#"           {up} (up)
           |   {north} (north)
           |  /
           | /
   {west} ______|/
(west)"#
    )
}

fn down_west_south(down: &str, south: &str, west: &str) -> String {
    format!(
        r#"   {west} ______
(west)    /|
         / |
        /  |
       {south}   {down} (down)
    (south)"#
    )
}

fn down_east_south(down: &str, south: &str, east: &str) -> String {
    format!(
        r#"        ______ {east} (east)
      /|
     / |
    /  |
   {south}   {down} (down)
(south)"#
    )
}

fn down_east_north(down: &str, north: &str, east: &str) -> String {
    format!(
        r#"    {north} (north)
   /
  /
 /______ {east} (east)
|
|
|
|
{down} (down)"#
    )
}

fn down_west_north(down: &str, north: &str, west: &str) -> String {
    format!(
        r#"            {north} (north)
           /
          /
{west} ______ /
(west)  |
        |
        |
        |
        {down} (down)
"#
    )
}

fn ascii_art(
    up_down: &str,
    north_south: &str,
    east_west: &str,
    up_down_axis: &str,
    north_south_axis: &str,
    east_west_axis: &str,
) -> String {
    match up_down {
        "up" => match north_south {
            "north" => match east_west {
                "east" => up_east_north(up_down_axis, north_south_axis, east_west_axis),
                "west" => up_west_north(up_down_axis, north_south_axis, east_west_axis),
                _ => unreachable!(),
            },
            "south" => match east_west {
                "east" => up_east_south(up_down_axis, north_south_axis, east_west_axis),
                "west" => up_west_south(up_down_axis, north_south_axis, east_west_axis),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        },
        "down" => match north_south {
            "north" => match east_west {
                "east" => down_east_north(up_down_axis, north_south_axis, east_west_axis),
                "west" => down_west_north(up_down_axis, north_south_axis, east_west_axis),
                _ => unreachable!(),
            },
            "south" => match east_west {
                "east" => down_east_south(up_down_axis, north_south_axis, east_west_axis),
                "west" => down_west_south(up_down_axis, north_south_axis, east_west_axis),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}
