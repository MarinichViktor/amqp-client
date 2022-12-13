use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote};
use syn::{Ident, Type, Attribute, punctuated::Punctuated, Token, Visibility, braced, parse_macro_input, Lit};
use syn::parse::{Parse, ParseStream};

pub static FIELD_ANNOTATION_WHITELIST: [&str; 13] = [
  "byte",
  "bool",
  "short",
  "ushort",
  "int",
  "uint",
  "long",
  "ulong",
  "float",
  "double",
  "short_str",
  "long_str",
  "prop_table",
];

pub struct MethodItem {
  pub attrs: Vec<Attribute>,
  pub vis: Visibility,
  pub ident: Ident,
  pub fields: Vec<MethodItemField>
}

pub struct MethodItemField {
  pub vis: Visibility,
  pub ident: Ident,
  pub ty: Type,
  pub amqp_ty: Option<String>
}

impl Parse for MethodItem {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let attrs: Vec<Attribute> = input.call(Attribute::parse_outer)?;
    let vis: Visibility = input.parse()?;
    let _: Token![struct] = input.parse()?;
    let ident: Ident = input.parse()?;
    let content;
    braced!(content in input);
    let fields: Punctuated<MethodItemField, Token![,]> = content.parse_terminated(MethodItemField::parse)?;

    Ok(MethodItem {
      attrs,
      vis,
      ident,
      fields: fields.into_iter().collect(),
    })
  }
}

impl Parse for MethodItemField {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let attrs: Vec<Attribute> = input.call(Attribute::parse_outer)?;
    let vis: Visibility = input.parse()?;
    let ident: Ident = input.parse()?;
    let _: Token![:] = input.parse()?;
    let ty: Type = input.parse()?;

    let mut type_attr =  attrs.into_iter()
      .map(|attr| {
        return attr.path.segments.into_iter()
          .map(|segment| { segment.ident.to_string() })
          .collect::<Vec<String>>()
          .join(":")
          .to_string();
      })
      .collect::<Vec<String>>();

    let amqp_ty = if type_attr.len() > 0 {
      type_attr.pop()
    } else {
      None
    };

    Ok(MethodItemField {
      vis,
      ident,
      ty,
      amqp_ty
    })
  }
}

pub struct MethodItemMeta(pub i16,pub i16);

impl Parse for MethodItemMeta {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let props = input.parse_terminated::<(String, i16), Token![,]>(
      |input| -> syn::Result<(String, i16)> {
        let prop_name = input.parse::<Ident>()?.to_string();

        if ![ "c_id", "m_id"].contains(&prop_name.as_str()) {
          return Err(input.error("Unknown prop name"));
        }

        let _ = input.parse::<Token![=]>()?;
        let attr_val = match input.parse::<Lit>()? {
          Lit::Int(lit) => {
            lit.base10_parse::<i16>()
          }
          _ => Err(input.error("Invalid value for attribute prop"))
        }?;

        Ok((prop_name, attr_val))
      })?.into_iter().collect::<Vec<(String, i16)>>();

    let class_id = props.iter().find(|i| { i.0 == "c_id"}).unwrap();
    let method_id = props.iter().find(|i| { i.0 == "m_id"}).unwrap();

    Ok(MethodItemMeta(class_id.1,method_id.1))
  }
}
