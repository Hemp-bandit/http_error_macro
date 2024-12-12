use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(ImplHttpError)]
pub fn impl_http_error(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the enum
    let enum_name = &input.ident;

    // Check if the input is an enum
    if let Data::Enum(_) = input.data {
        // Generate the implementation code
        let output = quote! {

            use rs_service_util::response::ResponseBody;
            use actix_web::HttpResponse;
            use actix_web::{error,http::header::ContentType};
            use  actix_web::http::StatusCode;

            impl error::ResponseError for #enum_name {

              fn error_response(&self) -> HttpResponse {
                let rsp_data = ResponseBody::error(&self.to_string());
                HttpResponse::build(self.status_code())
                    .insert_header(ContentType::json())
                    .insert_header(("Access-Control-Allow-Origin", "*"))
                    .json(rsp_data)
                }

                fn status_code(&self) -> StatusCode {
                    StatusCode::INTERNAL_SERVER_ERROR
                }

            }
        };

        // Return the generated code as a token stream
        return output.into();
    } else {
        panic!("EnumMethods can only be derived for enums");
    }
}
