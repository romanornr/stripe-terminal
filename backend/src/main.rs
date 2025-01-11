use actix_web::{middleware, web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use stripe::{Client as StripeClient, Client, CreatePaymentIntent, Currency, ListPaymentIntents, PaymentIntent, PaymentIntentCaptureMethod, TerminalReader, StripeError, CreateTerminalConnectionToken, TerminalConnectionToken, ListTerminalReaders, ListTerminalLocations, TerminalLocation};

//======================//
//   Shared App State   //
//======================//

// State shared across all handlers, including the Stripe client
struct Appstate {
    stripe_client: Arc<StripeClient>
}

//======================//
//   Response Structs   //
//======================//

// ApiResponse is a generic struct to wrap any response data
#[derive(Serialize)]
struct ApiResponse<T> {
    data: Option<T>,
    error: Option<String>,
}

// ApiResponse implementation for success responses
impl<T: Serialize> ApiResponse<T> {
    fn success(data: T) -> HttpResponse {
        HttpResponse::Ok().json(ApiResponse {
            data: Some(data),
            error: None,
        })
    }
}

// ErrorResponse is a struct to wrap any error response
#[derive(Serialize)]
struct ErrorResponse {
    data: Option<String>,
    error: Option<String>,
}

// ErrorResponse implementation for error responses
impl ErrorResponse {
    fn new(message: &str, status: actix_web::http::StatusCode) -> HttpResponse {
        HttpResponse::build(status).json(ErrorResponse {
            data: None,
            error: Some(message.to_string()),
        })
    }
}

//=========================================//
//   Test Endpoint (verifies Stripe key)   //
//=========================================//

async fn test_endpoint() -> impl Responder {
    match env::var("STRIPE_SECRET_KEY") {
        Ok(key) => {
            // Mask the secret key in the response
            let safe_key = format!("{}...{}", &key[..7], &key[key.len()-4..]);
            ApiResponse::success(format!("Env loaded! Key: {}", safe_key))
        }
        Err(_) => {
            ErrorResponse::new(
                "Failed to load stripe secret key",
                actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    }
}

//===================================================//
//   Endpoint to create Stripe PaymentIntent (POST)   //
//===================================================//
#[derive(Deserialize)]
struct CreatePaymentIntentRequest {
    amount: i64,
    currency: String,
}

async fn create_payment_intent_handler(
    data: web::Data<Appstate>,
    req: web::Json<CreatePaymentIntentRequest>,
)-> impl Responder {
    let currency_enum = match req.currency.parse::<Currency>() {
        Ok(currency) => currency,
        Err(_) => {
            return ErrorResponse::new("Invalid currency", actix_web::http::StatusCode::BAD_REQUEST);
        }
    };
    // Prepare PaymentIntent creation parameters
    let mut create_params = CreatePaymentIntent::new(req.amount, currency_enum); 
    create_params.payment_method_types = Some(vec!["card_present".to_string()]);
    create_params.capture_method = Some(PaymentIntentCaptureMethod::Automatic);
    
    match PaymentIntent::create(&data.stripe_client, create_params).await {
        Ok(payment_intent) => { ApiResponse::success(payment_intent) }
        Err(err) => ErrorResponse::new(&format!("Failed to create payment intent: {}", err), actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
    }
}

//====================================================== //
//   Endpoint to get 10 most recentPaymentIntent (GET)   //
//=====================================================  //
// Get /get-recent-payment-intents
async fn get_payment_intent_handler(data: web::Data<Appstate>) -> impl Responder {
    let mut params = ListPaymentIntents::new();
    params.limit = Some(10);

    match PaymentIntent::list(&data.stripe_client, &params).await {
        Ok(payment_intents) => ApiResponse::success(payment_intents),
        Err(err) => ErrorResponse::new(&format!("Failed to list payment intents: {}", err), actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
    }
}

//==========================================//
//         Endpoint to get location ID      //
//==========================================//
// GET /get-location-id
async fn get_location_id_handler()-> impl  Responder {
    match env::var("LOCATION_ID") {
        Ok(location_id) => ApiResponse::success(serde_json::json!({ "location_id": location_id })),
        Err(_) => ErrorResponse::new("Failed to load location ID", actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
    }
}

//==========================================//
//         Endpoint connection token        //
//==========================================//
async fn connection_token_handler(data: web::Data<Appstate>) -> impl Responder {
    match TerminalConnectionToken::create(&data.stripe_client, CreateTerminalConnectionToken::default()).await {
        Ok(token) => ApiResponse::success(serde_json::json!({ "secret": token.secret })),
        Err(err) => ErrorResponse::new(&format!("Failed to create connection token: {err}"), actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
    }
}


async fn get_reader_id_handler(data: web::Data<Appstate>) -> impl Responder {
    let locations = TerminalLocation::list(&data.stripe_client, &ListTerminalLocations::new()).await;
    match locations {
        Ok(locations) => {
            let location_id = env::var("LOCATION_ID")
                .expect("LOCATION_ID must be in .env");

            // Filter the entire list to find exactly the one matching location_id
            let maybe_loc = locations.data.into_iter().find(|loc| loc.id.as_str() == location_id);

            match maybe_loc {
                Some(location) => ApiResponse::success(location.id),
                None => ErrorResponse::new(&format!("No location found with ID {location_id}"), actix_web::http::StatusCode::NOT_FOUND
                ),
            }
        }
        Err(err) => ErrorResponse::new(&format!("Failed to list terminal locations: {err}"), actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}

//==========================================//
//         Cancel reader action             //
//==========================================//

// This struct is ust for Actix to deserialize the inbound JSON
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct CancelActionRequest {
    reader_id: String,
}

#[derive(Serialize)]
struct CancelReaderActionResponse {
    reader_state: TerminalReader
}

// This is the empty payload that we send to Stripe (no fields)
#[derive(serde::Serialize, Default)]
struct CancelActionStripeRequest;

async fn cancel_action(client: &Client, reader_id: &str) -> Result<TerminalReader, StripeError> {
    let url = format!("/terminal/readers/{reader_id}/cancel_action");
    // Send an empty JSON body (no fields)
    client.post_form(&url, &CancelActionStripeRequest::default()).await
}

async fn cancel_action_handler(data: web::Data<Appstate>, request_body: web::Json<CancelActionRequest>) -> impl Responder {
    let reader_id = &request_body.reader_id;
    match cancel_action(&data.stripe_client, reader_id).await {
        Ok(updated_reader) => {
            HttpResponse::Ok().json(CancelReaderActionResponse {
                reader_state: updated_reader
            })
        }
        Err(err) => ErrorResponse::new(&format!("Failed to cancel action for reader `{reader_id}`: {err}"), actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
    }
}

// async fn cancel_action(client: &Client, reader_id: &str, params: &CancelActionRequest) -> Result<TerminalReader, StripeError> {
//     let url = format!("/terminal/readers/{}/cancel", reader_id);
//     client.post_form(&url, params).await
// }

// async fn cancel_action(client: &Client, reader_id: &str, params: &CancelActionRequest, ) -> Result<TerminalReader, StripeError> {
//     let url = format!("/terminal/readers/{reader_id}/cancel_action");
//     client.post_form(&url, params).await
// }
//

// GET /cancel-action
// stripe terminal readers cancel action(readerId)
// async fn cancel_action_handler(data: web::Data<Appstate>, query: web::Json<CancelActionRequest>, ) -> impl Responder {
//     //let reader_id = &query.reader_id;
//     //let cancel_req = CancelActionRequest::default();
//
//     match cancel_action(&data.stripe_client, reader_id).await {
//         Ok(reader) => ApiResponse::success(reader),
//         Err(err) => ErrorResponse::new( &format!("Failed to cancel action for reader `{reader_id}`: {err}"), actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
//     }
//
//     // match cancel_action(&data.stripe_client, reader_id, &cancel_req).await {
//     //     Ok(reader) => ApiResponse::success(reader),
//     //     Err(err) => ErrorResponse::new( &format!("Failed to cancel action for reader `{reader_id}`: {err}"), actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
//     // }
// }

//=====================//
//   Main Entry Point  //
//=====================//

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // load environment variables
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    //Test env var loading
    let stripe_key = match env::var("STRIPE_SECRET_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("Failed to load stripe secret key");
            std::process::exit(1);
        }
    };
    
    // Create a single async-stripe client, shared by all handlers
    let stripe_client = Arc::new(StripeClient::new(&stripe_key));
    
    // Prepare shared application state
    let app_state = web::Data::new(Appstate {
        stripe_client: stripe_client.clone(),
    });

println!("Server starting on port 4242...");
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            // Add logger middleware
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            // register routes
            .route("/test", web::get().to(test_endpoint))
            .route("/create-payment-intent", web::post().to(create_payment_intent_handler))
            .route("/get-location-id", web::get().to(get_location_id_handler))
            .route("/connection-token", web::post().to(connection_token_handler))
            .route("/get-recent-payment-intents", web::get().to(get_payment_intent_handler))
            .route("/cancel-action", web::post().to(cancel_action_handler))
            .route("/readers/id", web::get().to(get_reader_id_handler))
    })
        .bind(("127.0.0.1", 4242))?
        .run()
        .await
}