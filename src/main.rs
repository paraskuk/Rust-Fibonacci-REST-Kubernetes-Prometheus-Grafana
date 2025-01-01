use actix_files as fs;
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    middleware::Logger,
    web, App, HttpResponse, HttpServer, Responder,
};
use serde::Deserialize;
use std::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};

use fibonacci::fibonacci_iterative;
use log::{error, info};
use log4rs;
use prometheus::{
    gather, register_counter, register_gauge, register_histogram, Encoder, TextEncoder,
};

static STATIC_DIR: &str = "/usr/src/app/static";
static LOG4RS_CONFIG: &str = "/usr/src/app/log4rs.yaml";
const MAX_DAILY_REQUESTS: u32 = 1000;

#[derive(Deserialize)]
struct FibonacciInput {
    n: u32,
}

lazy_static::lazy_static! {
    static ref REQUEST_COUNTER: prometheus::Counter = register_counter!(
        "requests_total",
        "Total number of requests"
    ).unwrap();
    static ref REQUEST_HISTOGRAM: prometheus::Histogram = register_histogram!(
        "request_duration_seconds",
        "Request duration in seconds"
    ).unwrap();
    static ref ACTIVE_REQUESTS: prometheus::Gauge = register_gauge!(
        "active_requests",
        "Number of active requests"
    ).unwrap();
    static ref RESPONSE_SIZE_HISTOGRAM: prometheus::Histogram = register_histogram!(
        "response_size_bytes",
        "Response size in bytes"
    ).unwrap();
}

// Handler for calculating Fibonacci
async fn calculate_fibonacci(
    data: web::Query<FibonacciInput>,
    daily_request_count: web::Data<Mutex<u32>>,
) -> impl Responder {
    let n = data.n;
    info!("Received request to calculate Fibonacci for n = {}", n);

    let mut count = match daily_request_count.lock() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to acquire lock on daily_request_count: {}", e);
            return HttpResponse::InternalServerError().body("Internal server error");
        }
    };

    if *count >= MAX_DAILY_REQUESTS {
        return HttpResponse::TooManyRequests()
            .body("Daily request limit reached. Please try again tomorrow.");
    }
    *count += 1;

    // Increase the "active requests" gauge
    ACTIVE_REQUESTS.inc();

    // Start timing this request
    let timer = REQUEST_HISTOGRAM.start_timer();

    let result = fibonacci_iterative(n);
    info!("Calculated Fibonacci for n = {}: {}", n, result);

    // Count this request in Prometheus metrics
    REQUEST_COUNTER.inc();

    // Stop the timer
    timer.observe_duration();

    // Decrement "active requests"
    ACTIVE_REQUESTS.dec();

    // Observe response size
    let response_size = serde_json::to_string(&result).unwrap().len() as f64;
    RESPONSE_SIZE_HISTOGRAM.observe(response_size);

    HttpResponse::Ok().json(result)
}

// Handler for Prometheus metrics
async fn metrics() -> impl Responder {
    let encoder = TextEncoder::new();
    let metric_families = gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    HttpResponse::Ok()
        .content_type(encoder.format_type())
        .body(buffer)
}

// Custom default handler to serve index.html
async fn default_handler(req: ServiceRequest) -> Result<ServiceResponse, actix_web::Error> {
    let (http_req, _payload) = req.into_parts();
    match fs::NamedFile::open(format!("{}/index.html", STATIC_DIR)) {
        Ok(file) => {
            info!("Serving index.html from: {}", STATIC_DIR);
            Ok(ServiceResponse::new(
                http_req.clone(),
                file.into_response(&http_req),
            ))
        }
        Err(_) => {
            error!("index.html not found in: {}", STATIC_DIR);
            Ok(ServiceResponse::new(
                http_req,
                HttpResponse::NotFound().body("File not found"),
            ))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize log4rs
    if let Err(e) = log4rs::init_file(LOG4RS_CONFIG, Default::default()) {
        eprintln!("Error initializing log4rs: {}", e);
        std::process::exit(1);
    }

    info!("Starting Fibonacci server...");

    // Wrap the request count in an Actix Data<Mutex>
    let daily_request_count = web::Data::new(Mutex::new(0u32));

    // Set up a cron job to reset the daily request count at midnight (UTC)
    let scheduler = JobScheduler::new().await.unwrap();
    let daily_request_count_clone = daily_request_count.clone();

    scheduler
        .add(
            Job::new_async("0 0 * * * *", move |_uuid, _l| {
                let daily_request_count_clone = daily_request_count_clone.clone();
                Box::pin(async move {
                    let mut count = daily_request_count_clone.lock().unwrap();
                    *count = 0;
                    println!("Daily request count reset to 0");
                })
            })
                .unwrap(),
        )
        .await
        .unwrap();

    scheduler.start().await.unwrap();

    // Start the HTTP server
    HttpServer::new(move || {
        App::new()
            // Make daily_request_count available to handlers
            .app_data(daily_request_count.clone())
            .wrap(Logger::default())
            // Routes
            .route("/fibonacci", web::get().to(calculate_fibonacci))
            .route("/metrics", web::get().to(metrics))
            // Static file service
            .service(
                fs::Files::new("/", STATIC_DIR)
                    .index_file("index.html")
                    .default_handler(default_handler),
            )
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
