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
                fn handle_cron_event(metadata: self::preamble::fermyon::spin_cron::cron_types::Metadata) -> Result<(), self::preamble::fermyon::spin_cron::cron_types::Error> {
                    match super::#func_name(metadata.try_into().expect("cannot convert from Spin Cron payload")) {
                        Ok(()) => Ok(()),
                        Err(e) => {
                            eprintln!("{}", e);
                            Err(self::preamble::fermyon::spin_cron::cron_types::Error::Other(e.to_string()))
                        },
                    }
                }

            }
            impl From<self::preamble::fermyon::spin_cron::cron_types::Metadata> for ::spin_cron_sdk::Metadata {
                fn from(resp:self::preamble::fermyon::spin_cron::cron_types::Metadata ) -> Self  {
                   Self { timestamp: resp.timestamp}
                }
            }

            impl From<self::preamble::fermyon::spin_cron::cron_types::Error> for ::spin_cron_sdk::Error {
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
