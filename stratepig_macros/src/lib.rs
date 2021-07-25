use proc_macro::TokenStream;
use quote::quote;
use syn::AttributeArgs;
use syn::{parse_macro_input, DeriveInput, ItemStruct};

#[proc_macro_attribute]
pub fn server_packet(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs)
        .into_iter()
        .find(|_x| true)
        .unwrap();
    let input = parse_macro_input!(input as ItemStruct);
    let name = input.clone().ident;

    let expanded = quote! {
        #[derive(Serialize, Deserialize, Clone, Debug)]
        #input

        impl stratepig_core::PacketBody for #name {
            fn serialize(&self) -> Result<Vec<u8>, stratepig_core::Error> {
                match bincode::serialize::<Self>(&self) {
                    Ok(d) => Ok(d),
                    Err(e) => Err(stratepig_core::Error::InvalidData(e.to_string())),
                }
            }

            fn deserialize(_data: &[u8]) -> Result<Self, stratepig_core::Error> {
                panic!("Trying to deserialize a server packet!");
            }

            fn id(&self) -> u8 {
                #args
            }

            fn box_clone(&self) -> Box<dyn PacketBody> {
                Box::new((*self).clone())
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn client_packet(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs)
        .into_iter()
        .find(|_x| true)
        .unwrap();
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.clone().ident;

    let expanded = quote! {
        #[derive(Serialize, Deserialize, Clone, Debug)]
        #input

        impl stratepig_core::PacketBody for #name {
            fn serialize(&self) -> Result<Vec<u8>, stratepig_core::Error> {
                panic!("Trying to serialize a client packet!");
            }

            fn deserialize(data: &[u8]) -> Result<Self, stratepig_core::Error> {
                match bincode::deserialize::<Self>(data) {
                    Ok(p) => Ok(p),
                    Err(e) => Err(stratepig_core::Error::InvalidData(format!("{:?}", e))),
                }
            }

            fn id(&self) -> u8 {
                #args
            }

            fn box_clone(&self) -> Box<dyn PacketBody> {
                Box::new((*self).clone())
            }
        }
    };

    TokenStream::from(expanded)
}
