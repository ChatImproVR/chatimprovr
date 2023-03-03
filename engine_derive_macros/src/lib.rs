extern crate proc_macro;

use cimvr_engine_interface::pkg_namespace;
use cimvr_engine_interface::prelude::ComponentIdStatic;
use cimvr_engine_interface::ecs::Component;


use proc_macro::TokenStream;
use quote::quote;
use syn;

struct Rawr {
    pub name: String,
}



#[proc_macro_derive(Component)]
pub fn component_macro_derive(input: TokenStream) -> TokenStream{
    // Make a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_component_macro(&ast)
}

fn impl_component_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident; // get the name of the struct we're deriving for
    // Use quote macro to output rust code
    let generated_code = quote! {
        impl Component for #name {
            const ID: ComponentIdStatic = ComponentIdStatic::new(pkg_namespace!(#name), std::mem::size_of::<#name>());
        }
    };
    // Return the generated code
    generated_code.into()
}