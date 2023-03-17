extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Lit, LitStr, Meta, MetaNameValue};

#[proc_macro_derive(Component)]
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

#[proc_macro_derive(Message, attributes(locality))]
pub fn message_derive(input: TokenStream) -> TokenStream {
    // Parse input to a syntax tree
    let input: DeriveInput = parse_macro_input!(input);

    // Get the name of the struct being derived. We're gonna use it to make the package namespace
    let name = &input.ident;

    // Try to get the locality attribute, we'll slap it onto Locality::{locality_str}
    let locality_lit = input.attrs.iter().find_map(|attr| {
        if attr.path.is_ident("locality") {
            attr.parse_args::<LitStr>().ok()
        } else {
            None
        }
    });

    let locality = locality_lit
        .expect("Expected locality attribute. Example `#[locality(\"Local\")] or #[locality(\"Remote\")]")
        .value();

    let locality: proc_macro2::TokenStream = locality.parse().unwrap();

    let output = quote::quote! {
       impl Message for #name {
            const CHANNEL: ChannelIdStatic = ChannelIdStatic {
                id: pkg_namespace!(stringify!(#name)),
                locality: Locality::#locality,
            };
        }
    };
    output.into()
}
