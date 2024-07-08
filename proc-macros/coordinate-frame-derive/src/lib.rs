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
fn process_unit_enum(_name: Ident, data_enum: DataEnum) -> TokenStream {
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
            for component  in components {
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

            if let Fields::Unit = &variant.fields {
                quote! {
                    pub struct #variant_name <T>([T; 3]);

                    impl<T> #variant_name <T> {
                        #(#components_impl)*
                    }
                }
            } else {
                unreachable!("This branch is unreachable due to process_enum()");
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
