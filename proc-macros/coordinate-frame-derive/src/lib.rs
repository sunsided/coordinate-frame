use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DataEnum, DeriveInput, Fields, Ident};

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
    let impls = data_enum.variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        // Ignore the special "Other" variant.
        if variant_name == "Other" {
            quote! {}
        } else {
            let components = split_variant_name_into_components(&variant_name.to_string());

            // Implementations for each component.
            let mut components_impl = Vec::new();

            // Generate native accessors for the components.
            for (i, component) in components.iter().enumerate() {
                let component_name = format_ident!("{component}");
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
            for component  in components.iter() {
                let pair = MUTUALLY_EXCLUSIVE.iter().copied().find(|&pair| pair.contains(&component.as_str())).expect("Failed to identify component pair");
                let other = pair.iter().copied().find(|&other| !other.eq(component.as_str())).expect("Failed to find component's opposite direction");

                let component_name = format_ident!("{component}");
                let other_name = format_ident!("{other}");
                let doc_str = format!("Returns the _{other}_ component of this coordinate. This component is not a native axis of the coordinate frame and is derived from the [`{component}`](Self::{component}) component at runtime.");

                components_impl.push(quote! {
                    #[doc = #doc_str]
                    #[inline]
                    pub fn #other_name (&self) -> T  where T: Copy + SaturatingNeg<Output = T> {
                        let component = self . #component_name();
                        component.saturating_neg()
                    }
                });
            }

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

            quote! {
                pub struct #variant_name <T>([T; 3]);

                impl<T> #variant_name <T> {
                    /// The coordinate frame.
                    const COORDINATE_FRAME: #enum_name = #enum_name :: #variant_name;

                    #[doc = #new_doc]
                    pub const fn new(#first_component: T, #second_component: T, #third_component: T) -> Self {
                        Self([#first_component, #second_component, #third_component])
                    }

                    /// Consumes self and returns its inner value.
                    pub const fn into_inner(self) -> [T; 3] where T: Copy {
                        self.0
                    }

                    /// Returns the coordinate frame of this instance.
                    pub const fn coordinate_frame(&self) -> #enum_name {
                        Self::COORDINATE_FRAME
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
            }
        }
    });

    let expanded = quote! {
        #(#impls)*
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
