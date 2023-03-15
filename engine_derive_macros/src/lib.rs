use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(ComponentDerive)]
pub fn component_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input: DeriveInput = parse_macro_input!(input);

    // Get the name of the struct being derived
    let name = &input.ident;

    // Generate the implementation of the `Component` trait
    let gen = quote::quote! {
         impl Component for #name {
             // Define a constant `ID` that identifies the component type
             // Use the namespace of the current crate and the name of the struct
             const ID: &'static str = pkg_namespace!(stringify!(#name));
         }
    };

    // Convert the generated code back into tokens and return them
    gen.into()
}
