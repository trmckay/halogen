use proc_macro::TokenStream;
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
                    self.write_byte(b);
                }
                Ok(())
            }
        }
    };

    output.into()
}

/// Derive `core::fmt::Write` for a `CharacterDevice`
#[proc_macro_derive(Address)]
pub fn usize_struct_derive_addr(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, .. } = parse_macro_input!(input);

    quote! {
        impl Address for #ident {
            fn as_ptr<T>(self) -> *const T {
                self.0 as *const T
            }

            fn as_mut_ptr<T>(self) -> *mut T {
                self.0 as *mut T
            }
        }

        impl core::ops::Add<usize> for #ident {
            type Output = #ident;

            fn add(self, rhs: usize) -> #ident {
                #ident(self.0 + rhs)
            }
        }

        impl core::ops::Sub<usize> for #ident {
            type Output = #ident;

            fn sub(self, rhs: usize) -> #ident {
                #ident(self.0 - rhs)
            }
        }

        impl core::ops::Sub<#ident> for #ident {
            type Output = usize;

            fn sub(self, rhs: #ident) -> usize {
                self.0 - rhs.0
            }
        }

        impl From<usize> for #ident {
            fn from(addr: usize) -> #ident {
                #ident(addr)
            }
        }

        impl From<#ident> for usize {
            fn from(addr: #ident) -> usize {
                addr.0
            }
        }

        impl core::fmt::Display for #ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{:p}", self.as_ptr::<u8>())
            }
        }

        impl core::fmt::Debug for #ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{:p}", self.as_ptr::<u8>())
            }
        }
    }
    .into()
}
