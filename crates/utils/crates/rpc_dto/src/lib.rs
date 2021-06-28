extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

mod try_from_rpc;
mod try_into_rpc;

#[proc_macro_error]
#[proc_macro_derive(TryFromRpc, attributes(rpc))]
pub fn try_from_rpc_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    try_from_rpc::analysis(input).synthesis()
}

#[proc_macro_error]
#[proc_macro_derive(TryIntoRpc, attributes(rpc))]
pub fn try_into_rpc_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    try_into_rpc::analysis(input).synthesis()
}

mod attrs;
