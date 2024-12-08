use actix_web::{middleware, web, App, HttpResponse, HttpServer, Responder};
use actix_cors::Cors;
use dotenv::dotenv;
use serde::{Serialize};

// Define a struct to represent the response format
#[derive(Serialize)]
struct ApiResponse<T> {
    data: Option<T>,
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

    // Add a status code to the error response
    fn error(message: &str, status: actix_web::http::StatusCode) -> HttpResponse {
        HttpResponse::build(status).json(ApiResponse::<T> {
            data: None,
            error: Some(message.to_string()),
        })
    }
}

async fn test_endpoint() -> impl Responder {
    ApiResponse::success("Hello, world")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // load .env file
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

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