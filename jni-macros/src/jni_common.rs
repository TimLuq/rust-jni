
use std::iter::FromIterator;

pub fn jni_mangle(val: String) -> String {
    val.chars().map(|c| match c {
        '.' => StringOrChar::Char('_'),
        '_' => StringOrChar::String("_1"),
        '/' => StringOrChar::String("_1"),
        ';' => StringOrChar::String("_2"),
        '[' => StringOrChar::String("_3"),
        _   => if (c >= 'a' && c <= 'z')
               || (c >= 'A' && c <= 'Z')
               || (c >= '0' && c <= '9') {
                    StringOrChar::Char(c)
               } else {
                    let hex = format!("{:x}", c as u16);
                    let zfill: String = std::iter::repeat('0').take(4 - hex.len()).collect();
                    StringOrChar::StringDyn(format!("_0{}{}", zfill, hex))
               }
    }).collect()
}

#[derive(Debug)]
enum StringOrChar {
    StringDyn(String),
    String(&'static str),
    Char(char),
}

impl FromIterator<StringOrChar> for String {
    fn from_iter<I: IntoIterator<Item=StringOrChar>>(iter: I) -> Self {
        let mut s = String::new();

        for i in iter {
            match i {
                StringOrChar::StringDyn(x) => s.push_str(&x),
                StringOrChar::String(x) => s.push_str(x),
                StringOrChar::Char(x) => s.push(x),
            }
        }

        s
    }
}

pub trait IntoPath {
    fn into_path(&self) -> (String, syn::PathArguments);
}

impl IntoPath for syn::PathSegment {
    fn into_path(&self) -> (String, syn::PathArguments) {
        (self.ident.to_string(), self.arguments.clone())
    }
}

impl IntoPath for String {
    fn into_path(&self) -> (String, syn::PathArguments) {
        (self.clone(), syn::PathArguments::None)
    }
}



pub fn ident_to_path(ident: &syn::Ident) -> syn::ExprPath {
    syn::ExprPath {
        attrs: Vec::new(),
        qself: Option::None,
        path: syn::Path {
            leading_colon: Option::None,
            segments: syn::punctuated::Punctuated::from_iter(std::iter::once(syn::PathSegment {
                ident: ident.clone(),
                arguments: syn::PathArguments::None,
            })),
        },
    }
}
pub fn ident_to_path_expr(ident: &syn::Ident) -> syn::Expr {
    ident_to_path(ident).into()
}
