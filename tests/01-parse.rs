// This test looks for a derive macro with the right name to exist. For now the
// test doesn't require any specific code to be generated by the macro, so
// returning an empty TokenStream should be sufficient.
//
// Before moving on, have your derive macro parse the macro input as a
// syn::DeriveInput syntax tree.
//
// Spend some time exploring the syn::DeriveInput struct on docs.rs by clicking
// through its fields in the documentation to see whether it matches your
// expectations for what information is available for the macro to work with.
//
//
// Resources:
//
//   - The Syn crate for parsing procedural macro input:
//     https://github.com/dtolnay/syn
//
//   - The DeriveInput syntax tree which represents input of a derive macro:
//     https://docs.rs/syn/2.0/syn/struct.DeriveInput.html
//
//   - An example of a derive macro implemented using Syn:
//     https://github.com/dtolnay/syn/tree/master/examples/heapsize

use builders::Builder;

#[derive(Builder)]
pub struct Command {
    executable: String,
    args: Vec<String>,
    env: Vec<String>,
    current_dir: String,
}

fn main() {}
