//! Provides the `color_from_hex!` macro for converting RGB and RGBA hex integral literals
//! to a byte array (or any `impl From<[u8; 3|4]>`) at compile time.
//!
//! # Examples
//! ```
//! # use hex_colors::color_from_hex;
//!
//! // The macro can be used in const context
//! const COLOR: [u8; 3] = color_from_hex!("010203");
//! # fn main() {
//! assert_eq!(COLOR, [1, 2, 3]);
//!
//! // It understands both upper and lower hex values
//! assert_eq!(color_from_hex!(0xa1b2c3d4), [0xA1, 0xB2, 0xC3, 0xD4]);
//! assert_eq!(color_from_hex!(0xE5E69092), [0xE5, 0xE6, 0x90, 0x92]);
//! assert_eq!(color_from_hex!(0x0a0B0C), [10, 11, 12]);
//! # }
//! ```

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, LitInt,
};

struct HexLit {
    pub repr: LitInt,
    pub bytes: usize,
}

impl Parse for HexLit {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let repr = input.parse::<LitInt>()?;

        if !repr.suffix().is_empty() {
            return Err(input.error("{integral} literal cannot be contains type suffix"));
        }

        let token = repr.token().to_string();
        let lit = match token.as_bytes() {
            [b'0', b'x', ..] => &token[2..],
            _ => return Err(input.error("expected `0x` prefix")),
        };

        let bytes = match lit.len() {
            bytes @ (6 | 8) => bytes / 2,
            _ => return Err(input.error("expected a maximum of 6 or 8 characters, ex: 4c4c4cff")),
        };

        Ok(Self { repr, bytes })
    }
}

/// Macro for converting a string literal containing hex-encoded color data
/// into an array of bytes.
#[proc_macro]
pub fn color_from_hex(input: TokenStream) -> TokenStream {
    let HexLit { repr, bytes } = parse_macro_input!(input as HexLit);

    let repr: proc_macro2::TokenStream = format!("{repr}_usize").parse().unwrap();

    if bytes == 3 {
        quote!([
            ((#repr & 0b1111_1111 << 16) >> 16) as u8,
            ((#repr & 0b1111_1111 << 8) >> 8) as u8,
            ((#repr & 0b1111_1111 << 0) >> 0) as u8,
        ])
    } else {
        quote!([
            ((#repr & 0b1111_1111 << 24) >> 24) as u8,
            ((#repr & 0b1111_1111 << 16) >> 16) as u8,
            ((#repr & 0b1111_1111 << 8) >> 8) as u8,
            ((#repr & 0b1111_1111 << 0) >> 0) as u8,
        ])
    }
    .into()
}
