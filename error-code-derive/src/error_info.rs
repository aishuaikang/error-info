use darling::{
    ast::{Data, Fields},
    util, FromDeriveInput, FromVariant,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(error_info))]
struct EnumFromDarling {
    ident: syn::Ident,
    generics: syn::Generics,
    data: Data<EnumFieldsInfo, ()>,

    app_type: syn::Type,
    prefix: String,
}

#[derive(Debug, FromVariant)]
#[darling(attributes(error_info))]
struct EnumFieldsInfo {
    ident: syn::Ident,
    fields: Fields<util::Ignored>,
    #[darling(default)]
    code: String,
    #[darling(default)]
    app_code: String,
    #[darling(default)]
    client_msg: String,
}

pub(crate) fn process_error_info(input: DeriveInput) -> TokenStream {
    let EnumFromDarling {
        ident,
        generics,
        data: Data::Enum(data),
        app_type,
        prefix,
    } = EnumFromDarling::from_derive_input(&input).expect("Can not parse input")
    else {
        panic!("Only enum is supported");
    };

    let code = data.iter().map(|variant| {
        let var_ident = &variant.ident;
        let var_code = &variant.code;
        let var_app_code = &variant.app_code;
        let var_client_msg = &variant.client_msg;
        let fields = &variant.fields;

        let varint_code = match fields.style {
            darling::ast::Style::Tuple => quote! {#ident::#var_ident(..)},
            darling::ast::Style::Struct => quote! {#ident::#var_ident{..}},
            darling::ast::Style::Unit => quote! {#ident::#var_ident},
        };

        let code = format!("{}{}", prefix, var_code);

        quote! {
            #varint_code => {
                error_code::ErrorInfo::new(
                    #var_app_code,
                    #code,
                    #var_client_msg,
                    self,
                )
            }
        }
    });

    quote! {
        impl #generics error_code::ToErrorInfo for #ident #generics {
            type T = #app_type;

            fn to_error_info(&self) -> error_code::ErrorInfo<Self::T> {
                match self {
                    #(#code),*
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::error_info::process_error_info;

    use super::EnumFromDarling;
    use darling::FromDeriveInput;
    #[test]
    fn test_darling_data_struct() {
        let input = r#"
            #[derive(Debug, Error, ToErrorInfo)]
            #[error_info(app_type = "http::StatusCode", prefix = "01")]
            pub enum MyError {
                #[error("Invalid command: {0}")]
                #[error_info(code = "IC", app_code = "200")]
                InvalidCommand(String),

                #[error("Invalid argument: {0}")]
                #[error_info(code = "IA", app_code = "400", client_msg = "friendly message")]
                InvalidArgument(String),

                #[error("{0}")]
                #[error_info(code = "RE", app_code = "500")]
                RespError(#[from] std::io::Error),
            }
            "#;

        let input = syn::parse_str(input).unwrap();

        let info = EnumFromDarling::from_derive_input(&input).unwrap();

        println!("{:#?}", info);

        assert_eq!(info.ident.to_string(), "MyError");
        assert_eq!(info.prefix, "01");

        let code = process_error_info(input);

        println!("{:?}", code);
    }
}
