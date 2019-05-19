#![recursion_limit = "256"]
extern crate proc_macro;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields};

#[proc_macro_derive(Closure, attributes(vecsemantics, name, id))]
pub fn closure(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let ident = ast.ident;
    let generics = &ast.generics;
    let where_clause = &ast.generics.where_clause;

    let register_fn = match ast.data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref _fields) => {
                let mut closure_name = Option::<String>::None;
                let mut closure_id = Option::<i32>::None;

                for attr in ast.attrs {
                    let meta = attr.parse_meta().unwrap();
                    match meta {
                        syn::Meta::NameValue(mnv) => {
                            if mnv.ident == "name" {
                                match mnv.lit {
                                    syn::Lit::Str(ls) => closure_name = Some(ls.value()),
                                    _ => (),
                                }
                            } else if mnv.ident == "id" {
                                match mnv.lit {
                                    syn::Lit::Int(li) => closure_id = Some(li.value() as i32),
                                    _ => (),
                                }
                            }
                        }
                        _ => (),
                    }
                }

                if closure_name.is_none() {
                    panic!("Must have a closure name");
                }

                if closure_id.is_none() {
                    panic!("Must have a closure id");
                }

                let expanded_fields = data
                    .fields
                    .iter()
                    .map(|f| generate_field(f))
                    .collect::<Vec<_>>();

                quote! {
                    let mut closure_params = Vec::new();
                    let mut offset = 0;

                    #(#expanded_fields)*

                    // finish
                    closure_params.push(ClosureParam {
                        typedesc: typedesc::UNKNOWN,
                        offset: std::mem::size_of::<#ident>(),
                        key: None,
                        field_size: std::mem::align_of::<#ident>(),
                    });

                    ss.register_closure(#closure_name,  #closure_id, &closure_params);
                }
            }
            _ => panic!("Can only work on named fields"),
        },
        _ => panic!("Can only work on structs"),
    };

    let expanded = quote! {
        impl #ident #generics #where_clause {
            pub fn register_with(ss: &mut ShadingSystem) {
                #register_fn
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

fn generate_field(field: &Field) -> TokenStream {
    let field_type = &field.ty;
    let typename = match field_type {
        syn::Type::Path(tp) => tp.path.segments[0].ident.to_string(),
        _ => panic!("asdajksdhaskd"),
    };

    let mut field_typedesc = match typename.as_str() {
        "Ustring" => quote! {typedesc::STRING},
        "f32" => quote! {typedesc::FLOAT},
        "V3f32" => quote! {typedesc::VECTOR},
        "i32" => quote! {typedesc::INT32},
        _ => panic!("Unsupported type: {:?}", field_type),
    };

    for attr in &field.attrs {
        let meta = attr.parse_meta().unwrap();
        match meta {
            syn::Meta::NameValue(mnv) => {
                if mnv.ident == "vecsemantics" {
                    match mnv.lit {
                        syn::Lit::Str(ls) => match ls.value().as_str() {
                            "VECTOR" => field_typedesc = quote! {typedesc::VECTOR},
                            "NORMAL" => field_typedesc = quote! {typedesc::NORMAL},
                            "COLOR" => field_typedesc = quote! {typedesc::COLOR},
                            "POINT" => field_typedesc = quote! {typedesc::POINT},
                            _ => (),
                        },
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }

    quote! {
        // #field_ident: #field_type
        closure_params.push(ClosureParam {
            typedesc: #field_typedesc,
            offset,
            key: None,
            field_size: std::mem::size_of::<#field_type>(),
        });
        offset += std::mem::size_of::<#field_type>();
    }
}
