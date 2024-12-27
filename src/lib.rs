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

#[macro_export]
macro_rules! http_client {
    () => {{
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_str("application/json;charset=utf8").unwrap(),
        );
        headers.insert(
            "service_call",
            reqwest::header::HeaderValue::from_str("store_service").unwrap(),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("build client faille");
        client
    }};
}

///
/// 获取 redis_conn
///
/// ```
///let mut rds = redis_conn!().await;
///
/// let res: Option<()> = rds.get("key").await.expect("msg");
///
/// ```
///
#[macro_export]
macro_rules! redis_conn {
    () => {
        async {
            let rds = crate::REDIS.get().expect("msg");
            rds.conn.clone()
        }
    };
}

///
/// 获取 transaction
/// *注意*: 在使用后尽快drop tx，否则在多个函数调用中多次调用 `RB.acquire()`  会卡住
///
/// ```
///let tx = transaction!().await;
///
/// let res: Option<()> = tx.query_decode("sql", []).await.expect("msg");
///
/// drop(tx);
///
/// ```
///
#[macro_export]
macro_rules! transaction {
    () => {
        async {
            let tx = crate::RB.acquire_begin().await.unwrap();
            let tx = tx.defer_async(|ex| async move {
                if ex.done() {
                    log::info!("transaction [{}] complete.", ex.tx_id);
                } else {
                    let r = ex.rollback().await;
                    if let Err(e) = r {
                        log::error!("transaction [{}] rollback fail={}", ex.tx_id, e);
                    } else {
                        log::info!("transaction [{}] rollback", ex.tx_id);
                    }
                }
            });
            log::info!("transaction [{}] start", tx.tx_id());
            tx
        }
    };
}
