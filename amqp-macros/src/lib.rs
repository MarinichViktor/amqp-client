use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote};
use syn::{Ident, Type, Attribute, punctuated::Punctuated, Token, Visibility, braced, parse_macro_input, Lit};
use syn::parse::{Parse, ParseStream};

mod macros;

static FIELD_ANNOTATION_WHITELIST: [&str; 13] = [
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

struct MethodItem {
  pub attrs: Vec<Attribute>,
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

struct MethodItemMeta(i16,i16);

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

  let into_byte_trait_impl: proc_macro2::TokenStream = generate_into_byte_vec_trait_impl(&method);
  let from_byte_trait_impl: proc_macro2::TokenStream = generate_from_byte_vec_trait_impl(&method);
  let meta_attrs: proc_macro2::TokenStream = generate_meta(&method, &meta);

  TokenStream::from(quote!(
    #(#struct_attrs)*
    #struct_vis struct #struct_name {
      #(#fields_def),*
    }

    #into_byte_trait_impl
    #from_byte_trait_impl
    #meta_attrs
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
          amqp_protocol::enc::Encode::write_short(&mut data, #struct_ident::class_id())?;
          amqp_protocol::enc::Encode::write_short(&mut data, #struct_ident::method_id())?;

          #( #fields_to_byte );*

          Ok(data)
        }
    }
  }
}

fn generate_field_to_byte(field: &MethodItemField) -> syn::Result<proc_macro2::TokenStream> {
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

fn generate_from_byte_vec_trait_impl(method: &MethodItem) -> proc_macro2::TokenStream {
  let fields_from_byte: Vec<proc_macro2::TokenStream> = method.fields.iter()
    .map(|field| {
      generate_field_from_byte(field).unwrap_or_else(syn::Error::into_compile_error)
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

fn generate_field_from_byte(field: &MethodItemField) -> syn::Result<proc_macro2::TokenStream> {
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

fn generate_meta(method: &MethodItem, meta: &MethodItemMeta) -> proc_macro2::TokenStream {
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
  let class_id = meta.0;
  let method_id = meta.1;

  quote! {
    impl #struct_ident {
      pub fn new( #(#constructor_args),* ) -> Self {
        Self {
          #(#field_idents),*
        }
      }

      fn class_id() -> i16 {
        #class_id
      }

      fn method_id() -> i16 {
        #method_id
      }
    }
  }
}
