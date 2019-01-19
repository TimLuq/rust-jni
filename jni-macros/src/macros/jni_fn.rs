extern crate syn;
use syn::{ItemFn};
use syn::export::{Span};

extern crate proc_macro;
use proc_macro::{TokenStream};

extern crate quote;
use quote::{quote};

use std::iter::Iterator;
use std::iter::FromIterator;

pub fn jni_fn(prefix: &str, refitem: &ItemFn) -> TokenStream {
    let mut errs = 0;
    match refitem.asyncness {
        Option::None => (),
        Option::Some(_) => {
            // TODO: Support async
            let err = "`async` is currently not supported for JNI functions";
            proc_macro::Span::call_site().error(err).emit();
            errs += 1;
        },
    };

    match refitem.abi {
        Option::None => (),
        Option::Some(_) => {
            let err = "let the JNI attribute handle `extern`";
            proc_macro::Span::call_site().error(err).emit();
            errs += 1;
        },
    };

    match refitem.decl.variadic {
        Option::None => (),
        Option::Some(_) => {
            // MAYBE: Support variadic? Needs common use cases and research.
            let err = "variadic functions are currently not supported for JNI functions";
            proc_macro::Span::call_site().error(err).emit();
            errs += 1;
        },
    };

    match refitem.vis {
        syn::Visibility::Public(_) => (),
        _ => {
            let err = "JNI function must be public";
            proc_macro::Span::call_site().error(err).emit();
            errs += 1;
        },
    };

    let orig_funcname = refitem.ident.to_string();
    let mut funcname = format!("Java_{}_{}", prefix, crate::jni_common::jni_mangle(orig_funcname));
    let refdecl: &syn::FnDecl = &refitem.decl;
    let mut stmts: Vec<syn::Stmt> = Vec::new();

    let mut inputs: syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma> = syn::punctuated::Punctuated::new();
    let mut passed_inputs: Vec<syn::Ident> = Vec::new();
    let mut output: syn::ReturnType = syn::ReturnType::Default;
    let mut ret_conv: Option<syn::Expr> = Option::None;
    let jninamestr0: String = format!("__jni__{}", funcname);
    let jniname0 = syn::Ident::new(&jninamestr0, Span::call_site());

    inputs.push(parse_quote!(env: #jniname0::JNIEnv));
    inputs.push(parse_quote!(obj: #jniname0::objects::JObject));

    // TODO: update inputs with type information from refitem arguments
    for i in &refdecl.inputs {
        match i {
            syn::FnArg::Captured(a) => {
                
            },
            _ => {
                let err = "explicit non-self type needed for input arguments";
                proc_macro::Span::call_site().error(err).emit();
                errs += 1;
            },
        }
    }

    let mut jninamestr: String = jninamestr0.clone();

    let jniname = syn::Ident::new(&jninamestr, Span::call_site());
    let mut jnipkg: syn::punctuated::Punctuated<syn::PathSegment, Token![::]> = syn::punctuated::Punctuated::new();
    jnipkg.push(syn::PathSegment { ident: jniname.clone(), arguments: syn::PathArguments::None });
    let ident_val = syn::Ident::new("val", Span::call_site());
    let ident_path = crate::jni_common::ident_to_path_expr(&ident_val);

    // TODO: generate all casting statements for inputs

    match &refdecl.output {
        syn::ReturnType::Default => (),
        syn::ReturnType::Type(a, t) => {
            match crate::types::ret::cast_to_java_out(&jniname, &t, &mut ret_conv) {
                Ok(ct) => {
                    output = syn::ReturnType::Type(a.clone(), Box::from(ct));
                },
                Err(err) => {
                    proc_macro::Span::call_site().error(err).emit();
                    errs += 1;
                },
            };
        },
    };

    if errs == 0 && passed_inputs.len() != refdecl.inputs.len() {
        let err = "somehow passed_inputs.len() != refdecl.inputs.len()";
        proc_macro::Span::call_site().error(err).emit();
        errs += 1;
    }

    if errs == 0 {
        let fcall: syn::ExprCall = syn::ExprCall {
            attrs: Vec::new(),
            func: Box::from(crate::jni_common::ident_to_path_expr(&refitem.ident)),
            paren_token: syn::token::Paren::default(),
            args: syn::punctuated::Punctuated::from_iter(passed_inputs.iter().map(|x| crate::jni_common::ident_to_path_expr(x))),
        };
        match output {
            syn::ReturnType::Default => {
                stmts.push(syn::Stmt::Semi(fcall.into(), syn::token::Semi::default()));
            },
            syn::ReturnType::Type(_, _) => {
                let letb: syn::Pat = (syn::PatIdent {
                    by_ref: Option::None,
                    mutability: Option::None,
                    ident: ident_val.clone(),
                    subpat: Option::None,
                }).into();
                stmts.push(syn::Stmt::Local(syn::Local {
                    attrs: Vec::new(),
                    let_token: syn::token::Let::default(),
                    pats: syn::punctuated::Punctuated::from_iter(std::iter::once(letb)),
                    ty: Option::None,
                    init: Option::Some((syn::token::Eq::default(), Box::new(fcall.into()))),
                    semi_token: syn::token::Semi::default(),
                }));
                match ret_conv {
                    Option::None => {
                        stmts.push(syn::Stmt::Expr(ident_path));
                    },
                    Option::Some(stmt) => {
                        stmts.push(syn::Stmt::Expr(stmt));
                    }
                }
            },
        };
        // TODO: generate statment for calling original function
    } else {
        // stmts.push(parse_quote!(compiler!("Previous errors occured")));
        stmts.push(parse_quote!(panic!("Compilation errors occured")));
    }


    let decl = Box::from(syn::FnDecl {
        fn_token: syn::token::Fn::default(),
        generics: syn::Generics::default(),
        paren_token: syn::token::Paren::default(),
        inputs,
        variadic: Option::None,
        output,
    });
    let block = Box::from(syn::Block {
        brace_token: refitem.block.brace_token,
        stmts,
    });

    let item = ItemFn {
        attrs: Vec::new(),
        vis: refitem.vis.clone(),
        constness: None,
        unsafety: refitem.unsafety,
        asyncness: None,
        // abi: Option::Some(syn::parse(TokenStream::from_str("extern \"system\"").unwrap()).unwrap()),
        abi: Option::Some(syn::Abi { extern_token: syn::token::Extern::default(), name: Option::Some(syn::LitStr::new("system", Span::call_site())) } ),
        // ident: syn::parse(TokenStream::from_str(&funcname).unwrap()).unwrap(),
        ident: syn::Ident::new(&funcname, Span::call_site()),
        decl,
        block,
    };

    let q = quote!{
        extern crate jni as #jniname ;
        #refitem
        #[no_mangle]
        #[allow(non_snake_case)]
        #item
    };

    TokenStream::from(q)
}
