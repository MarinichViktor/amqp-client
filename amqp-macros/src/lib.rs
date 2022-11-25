use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, Item, ext::IdentExt, Fields};

mod macros;

#[proc_macro_attribute]
pub fn amqp_method(meta: TokenStream, input: TokenStream) -> TokenStream {
  let backup =   proc_macro2::TokenStream::from(input.clone()).clone();
  let parsed: Item = parse(input).unwrap();
  let ind;
  match parsed {
      Item::Struct( i) => {
        println!("Identation {}", i.ident);
        ind = i.ident.unraw();
        i.fields.iter()
          .for_each(|f| {
            println!("Ident {:?}", f.ident);
            f.attrs.iter().for_each(|a| {
              let segs = a.path.segments.iter().map(|s| {
                return s.ident.to_string()
              }).collect::<Vec<String>>();
              println!("arttt {:?}", segs.first().unwrap());
            });
          });
      }
      _ => {
        panic!("Not matched");
      }
  }
  let q = quote!(
    #backup

    impl #ind {
      pub fn greet(&self) {
        println!("Greeting called");
      }
    }
  );
  TokenStream::from(q)
}
