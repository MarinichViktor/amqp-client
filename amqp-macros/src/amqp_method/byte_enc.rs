use proc_macro2::Span;
use quote::quote;
use syn::{Ident};
use crate::amqp_method::method_item::FIELD_ANNOTATION_WHITELIST;
use super::method_item::{MethodItem, MethodItemField};

pub fn imp_try_into_byte_vec(method: &MethodItem) -> proc_macro2::TokenStream {
  let fields_to_byte: Vec<proc_macro2::TokenStream> = method.fields.iter()
    .map(|field| {
      return field_to_byte_vec(field).unwrap_or_else(syn::Error::into_compile_error);
    })
    .collect();
  let struct_ident = &method.ident;

  quote! {
    impl TryInto<Vec<u8>> for #struct_ident {
        type Error = amqp_protocol::response::Error;

        fn try_into(self) -> amqp_protocol::response::Result<Vec<u8>, Self::Error> {
          let mut data = vec![];
          // amqp_protocol::enc::Encode::write_short(&mut data, #struct_ident::class_id())?;
          // amqp_protocol::enc::Encode::write_short(&mut data, #struct_ident::method_id())?;

          #( #fields_to_byte );*

          Ok(data)
        }
    }
  }
}

fn field_to_byte_vec(field: &MethodItemField) -> syn::Result<proc_macro2::TokenStream> {
  match &field.amqp_ty {
    Some(amqp_ty) => {
      let attr_ty_str = amqp_ty.as_str();
      return if FIELD_ANNOTATION_WHITELIST.contains(&attr_ty_str) {
        let field_ident = &field.ident;
        let attr_ident = Ident::new(format!("write_{}", attr_ty_str).as_str(), Span::call_site());

        Ok(quote! {
            amqp_protocol::enc::Encode::#attr_ident(&mut data, self.#field_ident)?;
          })
      } else {
        Err(syn::Error::new(
          field.ident.span(),
          format!(
            "Unsupported field attribute type: attribute '{}', field '{}'",
            amqp_ty.as_str(),
            field.ident
          ),
        ))
      }
    }
    _ => {
      Err(syn::Error::new(
        field.ident.span(),
        format!("Field type annotation required, field '{}'", field.ident),
      ))
    }
  }
}

pub fn imp_try_from_byte_vec(method: &MethodItem) -> proc_macro2::TokenStream {
  let fields_from_byte: Vec<proc_macro2::TokenStream> = method.fields.iter()
    .map(|field| {
      field_from_byte_vec(field).unwrap_or_else(syn::Error::into_compile_error)
    })
    .collect();
  let struct_ident = &method.ident;

  let fields_ident: Vec<proc_macro2::TokenStream> = method.fields.iter()
    .map(|field| {
      let field_ident = &field.ident;
      quote!{ #field_ident }
    })
    .collect();

  quote! {
    impl TryFrom<Vec<u8>> for #struct_ident {
        type Error = amqp_protocol::response::Error;

        fn try_from(data: Vec<u8>) -> amqp_protocol::response::Result<Self, Self::Error> {
          let mut data = std::io::Cursor::new(data);
          // skip class_id
          let _ = amqp_protocol::dec::Decode::read_short(&mut data)?;
          // skip method_id
          let _ = amqp_protocol::dec::Decode::read_short(&mut data)?;
          #( #fields_from_byte );*

          Ok(Self {
            #( #fields_ident ),*
          })
        }
    }
  }
}

fn field_from_byte_vec(field: &MethodItemField) -> syn::Result<proc_macro2::TokenStream> {
  match &field.amqp_ty {
    Some(amqp_ty) => {
      let attr_ty_str = amqp_ty.as_str();
      return if FIELD_ANNOTATION_WHITELIST.contains(&attr_ty_str) {
        let field_ident = &field.ident;
        let attr_ident = Ident::new(format!("read_{}", attr_ty_str).as_str(), Span::call_site());

        Ok(quote! {
            let #field_ident = amqp_protocol::dec::Decode::#attr_ident(&mut data)?;
          })
      } else {
        Err(syn::Error::new(
          field.ident.span(),
          format!(
            "Unsupported field attribute type: attribute '{}', field '{}'",
            amqp_ty.as_str(),
            field.ident
          ),
        ))
      }
    }
    _ => {
      Err(syn::Error::new(
        field.ident.span(),
        format!("Field type annotation required, field '{}'", field.ident),
      ))
    }
  }
}
