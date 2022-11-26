use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{Ident, Type,Attribute, punctuated::Punctuated, Token, Visibility, braced, parse_macro_input};
use syn::parse::{Parse, ParseStream};
use amqp_protocol::response;

mod macros;

struct MethodItem {
  pub vis: Visibility,
  pub ident: Ident,
  pub fields: Vec<MethodItemField>
}

struct MethodItemField {
  pub vis: Visibility,
  pub ident: Ident,
  pub ty: Type,
  pub amqp_ty: Option<String>
}

impl Parse for MethodItem {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let vis: Visibility = input.parse()?;
    let _: Token![struct] = input.parse()?;
    let ident: Ident = input.parse()?;

    // let fields = vec![];
    let content;
    braced!(content in input);
    let fields: Punctuated<MethodItemField, Token![,]> = content.parse_terminated(MethodItemField::parse)?;

    Ok(MethodItem {
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

#[proc_macro_attribute]
pub fn amqp_method(meta: TokenStream, input: TokenStream) -> TokenStream {
  let method: MethodItem = parse_macro_input!(input as MethodItem);
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

  let into_byte_trait_impl: proc_macro2::TokenStream = generate_into_byte_vec_trait_impl(&method);

  TokenStream::from(quote!(
    #struct_vis struct #struct_name {
      #(#fields_def),*
    }

    #into_byte_trait_impl
  ))
}

fn generate_into_byte_vec_trait_impl(method: &MethodItem) -> proc_macro2::TokenStream {
  let fields_to_byte: Vec<proc_macro2::TokenStream> = method.fields.iter()
    .map(|field| {
      return generate_field_to_byte(field).unwrap_or_else(syn::Error::into_compile_error);
    })
    .collect();
  let struct_ident = &method.ident;

  quote! {
    impl TryInto<Vec<u8>> for #struct_ident {
        type Error = amqp_protocol::response::Error;

        fn try_into(self) -> amqp_protocol::response::Result<Vec<u8>, Self::Error> {
          let mut data = vec![];

          #( #fields_to_byte );*

          Ok(data)
        }
    }
  }
}

fn generate_field_to_byte(field: &MethodItemField) -> syn::Result<proc_macro2::TokenStream> {
  match &field.amqp_ty {
    Some(amqp_ty) => {
      return match amqp_ty.as_str() {
        item @ "byte" => {
          let field_ident = &field.ident;
          let attr_ident = Ident::new(format!("write_{}", item).as_str(), Span::call_site());

          Ok(quote! {
            amqp_protocol::enc::Encode::#attr_ident(&mut data, self.#field_ident)?;
          })
        },
        _ => {
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
    }
    _ => {
      Err(syn::Error::new(
        field.ident.span(),
        format!("Field type annotation required, field '{}'", field.ident),
      ))
    }
  }
}
