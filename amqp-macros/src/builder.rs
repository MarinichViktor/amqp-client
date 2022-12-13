use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote};
use syn::{Ident, Type, Attribute, punctuated::Punctuated, Token, Visibility, braced, parse_macro_input, Lit};
use syn::parse::{Parse, ParseStream};

pub struct MethodItem {
  pub ident: Ident,
  pub fields: Vec<MethodItemField>
}

pub struct MethodItemField {
  pub vis: Visibility,
  pub ident: Ident,
  pub ty: Type,
}

impl Parse for MethodItem {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    // let _: Visibility = input.parse()?;
    let _: Token![struct] = input.parse()?;
    let ident: Ident = input.parse()?;
    let content;
    braced!(content in input);
    let fields: Punctuated<MethodItemField, Token![,]> = content.parse_terminated(MethodItemField::parse)?;

    Ok(MethodItem {
      ident,
      fields: fields.into_iter().collect(),
    })
  }
}

impl Parse for MethodItemField {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let vis: Visibility = input.parse()?;
    let ident: Ident = input.parse()?;
    let _: Token![:] = input.parse()?;
    let ty: Type = input.parse()?;

    Ok(MethodItemField {
      vis,
      ident,
      ty,
    })
  }
}

pub fn gen_builder_method(input: TokenStream) -> proc_macro2::TokenStream {
  let record: MethodItem = parse_macro_input!(input as MethodItem);

  let struct_ident = &record.ident;
  let fields = record.fields.iter()
    .map(|field| {
      let field_ident = &field.ident;
      let field_ty = &field.ty;
      quote! {
        pub fn #field_ident<'a>(&'a self, value: #field_ty) -> &'a Self {
          self.#field_ident = value;
          self
        }
      }
    }).collect::<proc_macro2::TokenStream>();

  quote! {
    impl #struct_ident {
      #( #fields ) *
    }
  }
}
