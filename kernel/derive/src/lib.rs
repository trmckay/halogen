use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive `core::fmt::Write` for a `CharacterDevice`
#[proc_macro_derive(ByteConsumerWrite)]
pub fn char_dev_derive_write(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);

    let output = quote! {
        impl core::fmt::Write for #ident {
            fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
                for b in s.bytes() {
                    if let Err(_) = self.write_byte(b) {
                        return Err(core::fmt::Error);
                    }
                }
                Ok(())
            }
        }
    };

    output.into()
}
