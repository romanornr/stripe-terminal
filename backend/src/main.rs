use actix_web::{middleware, web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use dotenv::dotenv;
use serde::{Serialize};
use std::env;
use std::fmt::format;

// Define a struct to represent the response format
#[derive(Serialize)]
struct ApiResponse<T> {
    data: Option<T>,
    error: Option<String>,
}

#[derive(Serialize)]
struct ErrorResponse {
    data: Option<String>,
    error: Option<String>,
}

// Implement a couple of helper methods for ApiResponse
impl<T: Serialize> ApiResponse<T> {
    fn success(data: T) -> HttpResponse {
        HttpResponse::Ok().json(ApiResponse {
            data: Some(data),
            error: None,
        })
    }
}

// Implement a helper method for ErrorResponse
impl ErrorResponse {
    fn new(message: &str, status: actix_web::http::StatusCode) -> HttpResponse {
        HttpResponse::build(status).json(ErrorResponse {
            data: None,
            error: Some(message.to_string()),
        })
    }
}

async fn test_endpoint() -> impl Responder {
    match env::var("STRIPE_SECRET_KEY") {
        Ok(key) => {
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // load .env file
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    //Test env var loading
    match env::var("STRIPE_SECRET_KEY") {
        Ok(_) => {
            println!("Stripe secret key loaded successfully");
        },
        Err(_) => {
            println!("Failed to load stripe secret key");
            std::process::exit(1);
        }
    }

println!("Server starting on port 4242...");
    HttpServer::new(|| {
        App::new()
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
    })
        .bind(("127.0.0.1", 4242))?
        .run()
        .await
}