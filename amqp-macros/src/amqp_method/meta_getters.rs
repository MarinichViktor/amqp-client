use quote::{quote};
use super::method_item::{MethodItem, MethodItemMeta};
use amqp_protocol::types::AmqpMethodArgs;

pub fn gen_method_getters(method: &MethodItem, meta: &MethodItemMeta) -> proc_macro2::TokenStream {
  let struct_ident = &method.ident;
  let class_id = meta.0;
  let method_id = meta.1;

  quote! {
    impl amqp_protocol::types::AmqpMethodArgs for #struct_ident {
      fn class_id(&self) -> i16 {
        #class_id
      }

      fn method_id(&self) -> i16 {
        #method_id
      }
    }
  }
}
