use actix_files as fs;
use actix_web::{web, App, HttpServer, Responder, HttpResponse, middleware::Logger};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use serde::Deserialize;
use fibonacci::fibonacci_iterative;
use log::{info, error};
use log4rs;
use prometheus::{Encoder, TextEncoder, register_counter, register_histogram, register_gauge,gather}; // Add prometheus imports

static STATIC_DIR: &str = "/usr/src/app/static"; // Absolute path
static LOG4RS_CONFIG: &str = "/usr/src/app/log4rs.yaml"; // Path to mounted log4rs.yaml

#[derive(Deserialize)]
struct FibonacciInput {
    n: u32,
}

// Register Prometheus metrics
lazy_static::lazy_static! {
    static ref REQUEST_COUNTER: prometheus::Counter = register_counter!("requests_total", "Total number of requests").unwrap();
    static ref REQUEST_HISTOGRAM: prometheus::Histogram = register_histogram!("request_duration_seconds", "Request duration in seconds").unwrap();
    static ref ACTIVE_REQUESTS: prometheus::Gauge = register_gauge!("active_requests", "Number of active requests").unwrap();
    static ref RESPONSE_SIZE_HISTOGRAM: prometheus::Histogram = register_histogram!("response_size_bytes", "Response size in bytes").unwrap();
}

async fn calculate_fibonacci(data: web::Query<FibonacciInput>) -> impl Responder {
    let n = data.n;
    if n < 0 {
        error!("Received negative number for Fibonacci calculation: {}", n);
        return HttpResponse::BadRequest().body("Negative numbers are not allowed");
    }
    info!("Received request to calculate Fibonacci for n = {}", n);

    // Increment the active requests gauge
    ACTIVE_REQUESTS.inc();

    // Start the timer before the calculation
    let timer = REQUEST_HISTOGRAM.start_timer();

    let result = fibonacci_iterative(n);
    info!("Calculated Fibonacci for n = {}: {}", n, result);

    // Increment the request counter and observe the request duration
    REQUEST_COUNTER.inc();
    timer.observe_duration(); // Stop the timer and observe the duration

    // Decrement the active requests gauge
    ACTIVE_REQUESTS.dec();

    // Observe the response size
    let response_size = serde_json::to_string(&result).unwrap().len() as f64;
    RESPONSE_SIZE_HISTOGRAM.observe(response_size);

    HttpResponse::Ok().json(result)
}

async fn metrics() -> impl Responder {
    let encoder = TextEncoder::new();
    let metric_families = gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    HttpResponse::Ok()
        .content_type(encoder.format_type())
        .body(buffer)
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
            .route("/metrics", web::get().to(metrics)) // Add metrics route
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