use proc_macro2::Span;
use syn::{Data, DeriveInput, Fields, GenericArgument, Path, PathArguments, Type, TypePath};

pub fn extract_fields(input: &DeriveInput) -> Result<&Fields, syn::Error> {
    fn generate_error(span: Span) -> syn::Error {
        syn::Error::new(
            span,
            "The 'Builder' macro only supports the 'struct' datatype",
        )
    }
    match &input.data {
        Data::Struct(data) => Ok(&data.fields),
        Data::Enum(data) => Err(generate_error(data.enum_token.span)),
        Data::Union(data) => Err(generate_error(data.union_token.span)),
    }
}

fn get_single_element_template_sub_type<'a>(
    ty: &'a Type,
    main_type_name: &str,
) -> Option<&'a Type> {
    match ty {
        Type::Path(TypePath {
            path: Path {
                segments,
                leading_colon,
            },
            ..
        }) if leading_colon.is_none()
            && segments.len() == 1
            && segments.iter().next().unwrap().ident == main_type_name =>
        {
            match &segments.iter().next().unwrap().arguments {
                PathArguments::AngleBracketed(generic_args) if generic_args.args.len() == 1 => {
                    let sub_arg = &generic_args.args.iter().next().unwrap();
                    match sub_arg {
                        GenericArgument::Type(sub_ty) => Some(sub_ty),
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}

pub fn get_option_sub_type(ty: &Type) -> Option<&Type> {
    get_single_element_template_sub_type(ty, "Option")
}

pub fn get_vec_sub_type(ty: &Type) -> Option<&Type> {
    get_single_element_template_sub_type(ty, "Vec")
}
