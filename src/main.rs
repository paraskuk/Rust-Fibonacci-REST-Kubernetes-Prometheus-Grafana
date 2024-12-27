use actix_files as fs;
use actix_web::{web, App, HttpServer, Responder, HttpResponse, middleware::Logger};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use serde::Deserialize;
use fibonacci::fibonacci_iterative;
use log::{info, error};
use log4rs;

static STATIC_DIR: &str = "/usr/src/app/static"; // Absolute path
static LOG4RS_CONFIG: &str = "/usr/src/app/log4rs.yaml"; // Path to mounted log4rs.yaml

#[derive(Deserialize)]
struct FibonacciInput {
    n: u32,
}

async fn calculate_fibonacci(data: web::Query<FibonacciInput>) -> impl Responder {
    let n = data.n;
    if n < 0 {
        error!("Received negative number for Fibonacci calculation: {}", n);
        return HttpResponse::BadRequest().body("Negative numbers are not allowed");
    }
    info!("Received request to calculate Fibonacci for n = {}", n);

    let result = fibonacci_iterative(n);
    info!("Calculated Fibonacci for n = {}: {}", n, result);

    HttpResponse::Ok().json(result)
}

async fn default_handler(req: ServiceRequest) -> Result<ServiceResponse, actix_web::Error> {
    let (http_req, _payload) = req.into_parts();
    match fs::NamedFile::open(format!("{}/index.html", STATIC_DIR)) {
        Ok(file) => {
            info!("Serving index.html from: {}", STATIC_DIR);
            Ok(ServiceResponse::new(http_req.clone(), file.into_response(&http_req)))
        },
        Err(_) => {
            error!("index.html not found in: {}", STATIC_DIR);
            Ok(ServiceResponse::new(http_req, HttpResponse::NotFound().body("File not found")))
        },
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging with log4rs
    if let Err(e) = log4rs::init_file(LOG4RS_CONFIG, Default::default()) {
        eprintln!("Error initializing log4rs: {}", e);
        std::process::exit(1);
    }

    info!("Starting Fibonacci server...");

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default()) // Add Logger middleware
            .route("/fibonacci", web::get().to(calculate_fibonacci))
            .service(
                fs::Files::new("/", STATIC_DIR)
                    .index_file("index.html")
                    .default_handler(default_handler)
            )
    })
        .bind("0.0.0.0:8080")? // Ensure binding to all interfaces
        .run()
        .await
}