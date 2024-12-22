use actix_web::{middleware, web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use stripe::{Client as StripeClient, CreatePaymentIntent, Currency, PaymentIntent, PaymentIntentCaptureMethod};

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
    create_params.payment_method_types = Some(vec!["card".to_string()]);
    create_params.capture_method = Some(PaymentIntentCaptureMethod::Automatic);
    
    match PaymentIntent::create(&data.stripe_client, create_params).await {
        Ok(payment_intent) => { ApiResponse::success(payment_intent) }
        Err(err) => ErrorResponse::new(&format!("Failed to create payment intent: {}", err), actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
    }
}

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
    })
        .bind(("127.0.0.1", 4242))?
        .run()
        .await
}