use proc_macro::TokenStream;
use quote::{quote};
use syn::{parse_macro_input};
use crate::amqp_method::factory::gen_method_factory;
use crate::amqp_method::meta_getters::gen_method_getters;
use amqp_method::method_item::{MethodItem, MethodItemMeta};
use crate::amqp_method::byte_enc::{imp_try_from_byte_vec, imp_try_into_byte_vec};

mod amqp_method;

#[proc_macro_attribute]
pub fn amqp_method(meta: TokenStream, input: TokenStream) -> TokenStream {
  let meta = parse_macro_input!(meta as MethodItemMeta);
  let method: MethodItem = parse_macro_input!(input as MethodItem);
  let struct_attrs: Vec<proc_macro2::TokenStream> = method.attrs.iter().map(|attr| {
    quote! {
      #attr
    }
  }).collect();
  let struct_vis = &method.vis;
  let struct_name = &method.ident;
  let fields_def = method.fields.iter().map(|f| {
    let vis = &f.vis;
    let ident = &f.ident;
    let ty = &f.ty;

    return quote! {
      #vis #ident: #ty
    }
  });

  let try_into_byte_vec_impl = imp_try_into_byte_vec(&method);
  let try_from_byte_vec_impl  = imp_try_from_byte_vec(&method);
  let method_factory = gen_method_factory(&method);
  let method_getters = gen_method_getters(&method, &meta);

  TokenStream::from(quote!(
    #(#struct_attrs)*
    #struct_vis struct #struct_name {
      #(#fields_def),*
    }

    #try_into_byte_vec_impl
    #try_from_byte_vec_impl
    #method_factory
    #method_getters
  ))
}
