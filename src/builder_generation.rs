use proc_macro2;

use super::builder_fields::BuilderField;
use super::builder_metadata::BuilderMetadata;

pub fn generate_builder_definition(
    builder_metadata: &BuilderMetadata,
    builder_fields: &Vec<BuilderField>,
) -> proc_macro2::TokenStream {
    let builder_struct_fields = &builder_fields
        .iter()
        .map(|field| field.generate_builder_struct_field())
        .collect::<Vec<_>>();
    let visibility = &builder_metadata.visibility;
    let builder_ident = &builder_metadata.builder_ident;
    quote! {
        #visibility struct #builder_ident {
            #(#builder_struct_fields),*
        }
    }
}

pub fn generate_struct_impl(
    builder_metadata: &BuilderMetadata,
    builder_fields: &Vec<BuilderField>,
) -> proc_macro2::TokenStream {
    let builder_default_fields = &builder_fields
        .iter()
        .map(|field| field.generate_builder_default_field())
        .collect::<Vec<_>>();
    let builder_ident = &builder_metadata.builder_ident;
    let struct_ident = &builder_metadata.struct_ident;
    quote! {
        impl #struct_ident {
            pub fn builder() -> #builder_ident {
                #builder_ident {
                    #(#builder_default_fields),*
                }
            }
        }
    }
}

pub fn generate_builder_impl(
    builder_metadata: &BuilderMetadata,
    builder_fields: &Vec<BuilderField>,
) -> proc_macro2::TokenStream {
    let builder_setters = &builder_fields
        .iter()
        .map(|field| field.generate_builder_setter())
        .collect::<Vec<_>>();
    let builder_iterative_setters = &builder_fields
        .iter()
        .map(|field| field.generate_builder_iterative_setter())
        .collect::<Vec<_>>();
    let build_property_replaces = &builder_fields
        .iter()
        .map(|field| field.generate_build_property_replace())
        .collect::<Vec<_>>();
    let props_setters = &builder_fields
        .iter()
        .map(|field| field.generate_prop_setter())
        .collect::<Vec<_>>();

    let builder_ident = &builder_metadata.builder_ident;
    let struct_ident = &builder_metadata.struct_ident;

    quote! {
        impl #builder_ident {
            #(#builder_setters)*

            #(#builder_iterative_setters)*

            pub fn build(&mut self) -> ::std::result::Result<#struct_ident, ::std::boxed::Box<dyn ::std::error::Error>> {
                #(#build_property_replaces);*;

                ::std::result::Result::Ok(#struct_ident{
                    #(#props_setters),*
                })
            }
        }
    }
}
