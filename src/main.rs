use actix_files as fs;
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use serde::Deserialize;
use fibonacci::fibonacci_iterative;
use log::info;

static STATIC_DIR: &str = "/usr/src/app/static"; // <--- Our absolute path

#[derive(Deserialize)]
struct FibonacciInput {
    n: u32,
}

async fn calculate_fibonacci(data: web::Query<FibonacciInput>) -> impl Responder {
    let n = data.n;
    let result = fibonacci_iterative(n);
    HttpResponse::Ok().json(result)
}

async fn default_handler(req: ServiceRequest) -> Result<ServiceResponse, actix_web::Error> {
    let (http_req, _payload) = req.into_parts();
    // Use the absolute path here:
    match fs::NamedFile::open(format!("{}/index.html", STATIC_DIR)) {
        Ok(file) => {
            info!("Serving index.html from: {}", STATIC_DIR);
            Ok(ServiceResponse::new(http_req.clone(), file.into_response(&http_req)))
        },
        Err(_) => {
            info!("index.html not found");
            Ok(ServiceResponse::new(http_req, HttpResponse::NotFound().body("File not found")))
        },
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            // Point the fs::Files to the absolute path
            .service(
                fs::Files::new("/", STATIC_DIR)
                    .index_file("index.html")
                    .default_handler(default_handler)
            )
            .route("/fibonacci", web::get().to(calculate_fibonacci))
    })
        .bind("0.0.0.0:8080")? // or 127.0.0.1:8080, but 0.0.0.0 is usually better in containers
        .run()
        .await
}
