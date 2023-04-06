use proc_macro::TokenStream as StdTokenStream;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields};

#[proc_macro_derive(Message)]
pub fn derive_message(input: StdTokenStream) -> StdTokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input as DeriveInput);

    match data {
        Data::Struct(_) => todo!(),
        Data::Enum(data) => {
            let encodes: TokenStream = data
                .variants
                .iter()
                .enumerate()
                .map(|(idx, variant)| {
                    let Ok(idx) = u8::try_from(idx) else {
                        unimplemented!("more than {} variants", u8::MAX);
                    };
                    let fields = match &variant.fields {
                        Fields::Named(fields) => {
                            let fields: TokenStream = fields
                                .named
                                .iter()
                                .enumerate()
                                .map(|(idx, field)| {
                                    let orig_name = field.ident.clone().unwrap();
                                    let ident = format_ident!("field{idx}");
                                    quote! {
                                        #orig_name: #ident,
                                    }
                                })
                                .collect();
                            quote! {
                                { #fields }
                            }
                        }
                        Fields::Unnamed(fields) => {
                            let fields: TokenStream = fields
                                .unnamed
                                .iter()
                                .enumerate()
                                .map(|(idx, _)| format_ident!("field{idx}").into_token_stream())
                                .collect();
                            quote! {
                                ( #fields )
                            }
                        }
                        Fields::Unit => TokenStream::new(),
                    };
                    let field_encodes: TokenStream = variant
                        .fields
                        .iter()
                        .enumerate()
                        .map(|(idx, field)| {
                            let ident = format_ident!("field{idx}");
                            let ty = &field.ty;
                            quote_spanned! {
                                field.span() =>
                                <#ty as message::Message>::encode(&#ident, &mut write)?;
                            }
                        })
                        .collect();
                    let ident = variant.ident.clone();
                    quote! {
                        Self::#ident #fields => {
                            #idx.encode(&mut write)?;
                            #field_encodes
                        },
                    }
                })
                .collect();
            let decodes: TokenStream = data
                .variants
                .iter()
                .enumerate()
                .map(|(idx, variant)| {
                    let Ok(idx) = u8::try_from(idx) else {
                        unimplemented!("more than {} variants", u8::MAX);
                    };
                    let fields = match &variant.fields {
                        Fields::Named(fields) => {
                            let fields: TokenStream = fields
                                .named
                                .iter()
                                .enumerate()
                                .map(|(idx, field)| {
                                    let orig_name = field.ident.clone().unwrap();
                                    let ident = format_ident!("field{idx}");
                                    quote! {
                                        #orig_name: #ident,
                                    }
                                })
                                .collect();
                            quote! {
                                { #fields }
                            }
                        }
                        Fields::Unnamed(fields) => {
                            let fields: TokenStream = fields
                                .unnamed
                                .iter()
                                .enumerate()
                                .map(|(idx, _)| format_ident!("field{idx}").into_token_stream())
                                .collect();
                            quote! {
                                ( #fields )
                            }
                        }
                        Fields::Unit => TokenStream::new(),
                    };
                    let field_decodes: TokenStream = variant
                        .fields
                        .iter()
                        .enumerate()
                        .map(|(idx, field)| {
                            let ident = format_ident!("field{idx}");
                            let ty = &field.ty;
                            quote_spanned! {
                                field.span() =>
                                let #ident = <#ty as message::Message>::decode(&mut read)?;
                            }
                        })
                        .collect();
                    let ident = variant.ident.clone();
                    quote! {
                        #idx => {
                            #field_decodes
                            Ok(Self::#ident #fields)
                        },
                    }
                })
                .collect();
            let invalid_str = format!("invalid {ident} id {{id}}");
            quote! {
                impl message::Message for #ident {
                    fn encode(&self, mut write: impl std::io::Write) -> std::io::Result<()> {
                        match self {
                            #encodes
                        }
                        write.flush()
                    }

                    fn decode(mut read: impl std::io::Read) -> std::io::Result<Self> {
                        let id = u8::decode(&mut read)?;
                        match id {
                            #decodes
                            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!(#invalid_str))),
                        }
                    }
                }
            }
            .into()
        }
        Data::Union(_) => todo!(),
    }
}
