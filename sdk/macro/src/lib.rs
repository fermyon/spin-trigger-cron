use proc_macro::TokenStream;
use quote::quote;

const WIT_PATH: &str = "../..";

#[proc_macro_attribute]
pub fn cron_component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(item as syn::ItemFn);
    let func_name = &func.sig.ident;
    let preamble = preamble();

    quote!(
        #func
        mod __spin_cron {
            mod preamble {
                #preamble
            }
            impl self::preamble::Guest for preamble::Cron {
                fn handle_cron_event(metadata: self::preamble::fermyon::spin_cron::cron_types::Metadata) -> ::std::result::Result<(), self::preamble::fermyon::spin_cron::cron_types::Error> {
                    match super::#func_name(::std::convert::TryInto::try_into(metadata).expect("cannot convert from Spin Cron payload")) {
                        ::std::result::Result::Ok(()) => ::std::result::Result::Ok(()),
                        ::std::result::Result::Err(e) => {
                            eprintln!("{}", e);
                            ::std::result::Result::Err(self::preamble::fermyon::spin_cron::cron_types::Error::Other(e.to_string()))
                        },
                    }
                }
            }
            impl ::std::convert::From<self::preamble::fermyon::spin_cron::cron_types::Metadata> for ::spin_cron_sdk::Metadata {
                fn from(metadata: self::preamble::fermyon::spin_cron::cron_types::Metadata) -> Self  {
                   Self { timestamp: metadata.timestamp}
                }
            }

            impl ::std::convert::From<self::preamble::fermyon::spin_cron::cron_types::Error> for ::spin_cron_sdk::Error {
                fn from(err: self::preamble::fermyon::spin_cron::cron_types::Error) -> Self {
                    match err {
                        self::preamble::fermyon::spin_cron::cron_types::Error::Other(e) => Self::Other(e)
                    }
                }
            }
        }
    )
        .into()
}

fn preamble() -> proc_macro2::TokenStream {
    let world = "spin-cron";
    quote! {
        #![allow(missing_docs)]
        ::spin_cron_sdk::wit_bindgen::generate!({
            world: #world,
            path: #WIT_PATH,
            runtime_path: "::spin_cron_sdk::wit_bindgen::rt",
            exports: {
                world: Cron
            }
        });
        pub struct Cron;
    }
}
