use quote::{quote};
use super::method_item::{MethodItem};

pub fn gen_method_factory(method: &MethodItem) -> proc_macro2::TokenStream {
  let constructor_args = method.fields.iter().map(|field| {
    let ident = &field.ident;
    let ty = &field.ty;

    quote! { #ident: #ty }
  }).collect::<Vec<proc_macro2::TokenStream>>();

  let field_idents = method.fields.iter().map(|field| {
    let ident = &field.ident;
    quote! { #ident }
  }).collect::<Vec<proc_macro2::TokenStream>>();

  let struct_ident = &method.ident;

  quote! {
    impl #struct_ident {
      pub fn new( #(#constructor_args),* ) -> Self {
        Self {
          #(#field_idents),*
        }
      }
    }
  }
}
