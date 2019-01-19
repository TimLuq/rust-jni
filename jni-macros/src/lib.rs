#![feature(proc_macro_diagnostic)]
#![feature(box_patterns)]
//macro_rules! jni_fn {
//    ($c:ident, $f:ident, $b:block) => {
//        #[no_mangle]
//        #[allow(non_snake_case)]
//        pub extern "system" fn Java_net_timluq_mc_rusticspigot_$c_$f $b
//    }
//}

#[macro_use]
extern crate syn;

extern crate proc_macro;
use proc_macro::{TokenStream};

#[macro_use]
extern crate quote;


mod jni_common;
mod types;
mod macros;

#[proc_macro_attribute]
pub fn jni(attr: TokenStream, input: TokenStream) -> TokenStream {
    let ats = jni_common::jni_mangle(attr.to_string().chars().filter(|&c| c != ' ').collect());
    let item: syn::Item = syn::parse(input).expect("jni attribute may only be used for module level items");
    match item {
        syn::Item::Fn(x) => macros::jni_fn::jni_fn(&ats, &x),
        // syn::Item::Impl(x) => jni_impl(&ats, &x),
        _ => panic!("jni attribute may only be placed on `fn` or `impl`"),
    }
}

