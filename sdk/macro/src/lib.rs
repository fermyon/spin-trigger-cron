use proc_macro::TokenStream;
use quote::quote;

const WIT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../cron.wit");

#[proc_macro_attribute]
pub fn cron_component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(item as syn::ItemFn);
    let func_name = &func.sig.ident;
    let await_postfix = func.sig.asyncness.map(|_| quote!(.await));
    let preamble = preamble();

    quote!(
        #func
        mod __spin_cron {
            mod preamble {
                #preamble
            }
            impl self::preamble::Guest for preamble::Cron {
                fn handle_cron_event(metadata: ::spin_cron_sdk::Metadata) -> ::std::result::Result<(), ::spin_cron_sdk::Error> {
                    ::spin_cron_sdk::executor::run(async move {
                        match super::#func_name(metadata)#await_postfix {
                            ::std::result::Result::Ok(()) => ::std::result::Result::Ok(()),
                            ::std::result::Result::Err(e) => {
                                eprintln!("{}", e);
                                ::std::result::Result::Err(::spin_cron_sdk::Error::Other(e.to_string()))
                            },
                        }
                    })
                }
            }
        }
    ).into()
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
            },
            with: {
                "fermyon:spin-cron/cron-types@2.0.0": ::spin_cron_sdk,
            }
        });
        pub struct Cron;
    }
}
