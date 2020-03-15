extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

// use urpc::server::Request;

#[proc_macro_derive(Request)]
pub fn request(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    if let syn::Data::Enum(data) = input.data {
        for variant in data.variants {
            println!(">>> {}", variant.ident);
            if let syn::Fields::Unnamed(fields) = variant.fields {
                for field in fields.unnamed {
                    println!(">>>> {:?}", field.ty);
                }
            }
        }
    }

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl<'a> Request<'a> for #name<'a> {
            fn from_bytes(header: urpc::RequestHeader, buf: &'a [u8]) -> urpc::server::Result<Self> {
                Err(urpc::server::Error::WontImplement)
            }
        }
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}
