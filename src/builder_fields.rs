use super::syntax_processing;
use proc_macro2::Span;
use syn::{
    spanned::Spanned, Attribute, Field, Fields, Ident, Lit, Meta, MetaNameValue, NestedMeta, Type,
};

const ITERATIVE_IDENT_ATTR_MACRO_NAME: &str = "builder";
const EACH_ATTR_NAME: &str = "each";

pub struct BuilderField<'a> {
    pub ident: &'a Ident,
    pub ty: &'a Type,
    pub iterative_name: Option<String>,
}

impl<'a> BuilderField<'a> {
    pub fn from_fields(fields: &'a Fields) -> Result<Vec<Self>, syn::Error> {
        let fields_named = match fields {
            Fields::Named(fields_named) => fields_named,
            _ => unimplemented!(),
        };
        let mut builder_fields = vec![];
        for field in &fields_named.named {
            builder_fields.push(BuilderField {
                ident: field.ident.as_ref().unwrap(),
                ty: &field.ty,
                iterative_name: extract_iterative_ident(field)?,
            })
        }
        Ok(builder_fields)
    }

    pub fn generate_builder_setter(&self) -> proc_macro2::TokenStream {
        let BuilderField {
            ident,
            ty,
            iterative_name,
        } = self;
        let param_type = match syntax_processing::get_option_sub_type(ty) {
            Some(sub_ty) => sub_ty,
            None => *ty,
        };
        match iterative_name {
            Some(_) => quote! {},
            None => quote! {
                fn #ident (&mut self, #ident: #param_type) -> &mut Self {
                    self.#ident = ::std::option::Option::Some(#ident);
                    self
                }
            },
        }
    }

    pub fn generate_builder_struct_field(&self) -> proc_macro2::TokenStream {
        let BuilderField { ident, ty, .. } = self;
        let new_ty = match syntax_processing::get_option_sub_type(ty) {
            Some(_) => quote! {#ty},
            None => quote! {::std::option::Option<#ty>},
        };
        quote! {#ident: #new_ty}
    }

    pub fn generate_builder_default_field(&self) -> proc_macro2::TokenStream {
        let BuilderField {
            ident,
            iterative_name,
            ..
        } = self;
        let default_value = match iterative_name {
            Some(_) => quote! {::std::option::Option::Some(vec![])},
            None => quote! {::std::option::Option::None},
        };
        quote! {#ident: #default_value}
    }

    pub fn generate_builder_iterative_setter(&self) -> proc_macro2::TokenStream {
        let BuilderField {
            ident,
            ty,
            iterative_name,
        } = self;
        match iterative_name {
            Some(iter_name) => {
                let param_ident = Ident::new(iter_name, Span::call_site());
                let param_type = syntax_processing::get_vec_sub_type(ty);
                match param_type {
                    Some(param_type) => quote! {
                        fn #param_ident (&mut self, #param_ident: #param_type) -> &mut Self {
                            self.#ident.as_mut().unwrap().push(#param_ident);
                            self
                        }
                    },
                    None => quote! {},
                }
            }
            None => quote! {},
        }
    }

    pub fn generate_build_property_replace(&self) -> proc_macro2::TokenStream {
        let BuilderField { ident, .. } = self;
        quote! {let #ident = ::std::mem::replace(&mut self.#ident, ::std::option::Option::None)}
    }

    pub fn generate_prop_setter(&self) -> proc_macro2::TokenStream {
        let BuilderField { ident, ty, .. } = self;
        let error_msg = format!("Error: field '{}' haven't been set.", ident);
        match syntax_processing::get_option_sub_type(ty) {
            Some(_) => quote! {#ident},
            None => quote! {
                #ident: match #ident {
                    ::std::option::Option::Some(value) => value,
                    ::std::option::Option::None => return ::std::result::Result::Err(
                        ::std::boxed::Box::new(::std::io::Error::new(::std::io::ErrorKind::InvalidInput, #error_msg)))
                }
            },
        }
    }
}

fn extract_iterative_ident(field: &Field) -> Result<Option<String>, syn::Error> {
    let attrs = field
        .attrs
        .iter()
        .filter(|attr| {
            let segments = &attr.path.segments;
            segments.len() == 1
                && segments.iter().next().unwrap().ident == ITERATIVE_IDENT_ATTR_MACRO_NAME
        })
        .collect::<Vec<&Attribute>>();
    if attrs.len() == 0 {
        return Ok(None);
    }
    let meta_list = match attrs.iter().next().unwrap().parse_meta() {
        Ok(Meta::List(meta_list)) => meta_list,
        _ => return Ok(None),
    };

    let (each_attribute, errors) = meta_list
        .nested
        .iter()
        .filter_map(|nested_meta| match nested_meta {
            NestedMeta::Meta(Meta::NameValue(meta_name_value)) => {
                match meta_name_value.path.segments.len() {
                    1 => Some(
                        match meta_name_value
                            .path
                            .segments
                            .iter()
                            .next()
                            .unwrap()
                            .ident
                            .to_string()
                            .as_str()
                        {
                            EACH_ATTR_NAME => Ok(meta_name_value),
                            _ => Err(syn::Error::new(
                                meta_name_value.path.segments.iter().next().unwrap().span(),
                                "expected `builder(each = \"...\")`",
                            )),
                        },
                    ),
                    _ => None,
                }
            }
            _ => None,
        })
        .partition::<Vec<Result<&MetaNameValue, syn::Error>>, _>(Result::is_ok);
    if errors.len() > 0 {
        return Err(errors.into_iter().next().unwrap().unwrap_err());
    }
    if each_attribute.len() != 1 {
        return Ok(None);
    }

    Ok(
        match &each_attribute.iter().next().unwrap().clone().unwrap().lit {
            Lit::Str(lit_str) => Some(lit_str.value()),
            _ => None,
        },
    )
}
