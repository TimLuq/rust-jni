
pub fn cast_from_java(jnipkg: &syn::Ident, typ: &syn::Type, ret: &mut Option<syn::Expr>) -> Result<(syn::Type, str), String> {
    
}
pub fn type_from_java(jnipkg: &syn::Ident, typ: &syn::Type) -> Result<(syn::Type, str), String> {
    match typ {
        syn::Type::Slice(t) => {
            cast_from_java_arr(jnipkg, &t.elem, ret)
        },
        syn::Type::Array(t) => {
            cast_from_java_out(jnipkg, &t.elem, ret)
        },
        syn::Type::Path(t) => {
            match t.qself {
                Option::Some(_) => Err("Self-qualified return types not allowed".to_string()),
                Option::None => match t.path.leading_colon {
                    Option::None => {
                        match t.path.segments.first() {
                            Option::Some(syn::punctuated::Pair::End(x)) => match x.into_path() {
                                (st, _) => match st.as_ref() {
                                    "i64" => Ok(parse_quote!(::#jnipkg::sys::jlong)),
                                    "i32" => Ok(parse_quote!(::#jnipkg::sys::jint)),
                                    "i16" => Ok(parse_quote!(::#jnipkg::sys::jshort)),
                                    "u16" => Ok(parse_quote!(::#jnipkg::sys::jchar)),
                                    "i8" => Ok(parse_quote!(::#jnipkg::sys::jbyte)),
                                    "u8" => {
                                        *ret = Option::Some(parse_quote!(val as i8));
                                        Ok(parse_quote!(::#jnipkg::sys::jbyte))
                                    },
                                    "bool" => {
                                        *ret = Option::Some(parse_quote!(val as u8));
                                        Ok(parse_quote!(::#jnipkg::sys::jboolean))
                                    },
                                    "str" => {
                                        *ret = Option::Some(parse_quote!(env.new_string(val).expect("Couldn't initialize a java string from return value.").into_inner()));
                                        Ok(parse_quote!(::#jnipkg::sys::jstring))
                                    }
                                    _ => Err(format!("JNI return types must be a primative or absolute with leading double colon, but found `{}`", st)),
                                },
                            },
                            _ => Err("JNI return types must be a primative or absolute with leading double colon".to_string()),
                        }
                    },
                    Option::Some(_) => cast_to_java_out_path(jnipkg, &mut t.path.segments.clone().into_iter().map(|x| x.into_path()), ret),
                },
            }
        },
        syn::Type::Paren(t) => {
            cast_to_java_out(jnipkg, &t.elem, ret)
        },
        syn::Type::Group(t) => {
            cast_to_java_out(jnipkg, &t.elem, ret)
        },
        _ => {
            Err("Unsupported returned data type.".to_string())
        }
    }
}
