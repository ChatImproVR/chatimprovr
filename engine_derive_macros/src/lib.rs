use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, LitInt};

#[proc_macro_derive(ComponentDerive, attributes(size))]
pub fn component_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input: DeriveInput = parse_macro_input!(input);

    // Get the name of the struct being derived
    let name = &input.ident;

    // Get the size attribute or fail at compile time
    let size_lit = input.attrs.iter().find_map(|attr| {
        if attr.path.is_ident("size") {
            attr.parse_args::<LitInt>().ok()
        } else {
            None
        }
    });

    let size = size_lit
        .expect("Expected size attribute. Example `#[size(12)]")
        .base10_parse::<u16>()
        .expect("Invalid size. Must be a u16.");

    // Generate the implementation of the `Component` trait
    let gen = quote::quote! {
        impl Component for #name {
            // Define a constant `ID` that identifies the component type
            const ID: ComponentIdStatic = ComponentIdStatic {
                // Use the namespace of the current crate and the name of the struct
                id: pkg_namespace!(stringify!(#name)),
                // Use the provided size
                size: #size,
            };
        }
    };

    // Convert the generated code back into tokens and return them
    gen.into()
}
