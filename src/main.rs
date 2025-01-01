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
    gather, register_counter, register_gauge, register_histogram, register_int_counter_vec,
    Encoder, TextEncoder, IntCounterVec,
};

static STATIC_DIR: &str = "/usr/src/app/static";
static LOG4RS_CONFIG: &str = "/usr/src/app/log4rs.yaml";

// Adjust this limit as desired
const MAX_DAILY_REQUESTS: u32 = 1001;

#[derive(Deserialize)]
struct FibonacciInput {
    n: u32,
}

lazy_static::lazy_static! {
    // Existing metrics
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

    // NEW METRIC #1: Tracks how many times the daily request limit was reached
    static ref REQUEST_LIMIT_REACHED_COUNTER: prometheus::Counter = register_counter!(
        "request_limit_reached_total",
        "Number of times the service returned TooManyRequests"
    ).unwrap();

    // NEW METRIC #2: A gauge to record the latest Fibonacci input (n)
    static ref LATEST_N_GAUGE: prometheus::Gauge = register_gauge!(
        "latest_fibonacci_input",
        "Most recent n value requested for Fibonacci"
    ).unwrap();

    // NEW METRIC #3: Tracks requests by status code (e.g., 200, 400, 429, 500, etc.)
    static ref REQUEST_STATUS_COUNTER: IntCounterVec = register_int_counter_vec!(
        "request_status_codes_total",
        "Number of requests by HTTP status code",
        &["code"]
    ).unwrap();
}

// Handler for calculating Fibonacci
async fn calculate_fibonacci(
    data: web::Query<FibonacciInput>,
    daily_request_count: web::Data<Mutex<u32>>,
) -> impl Responder {
    let n = data.n;
    info!("Received request to calculate Fibonacci for n = {}", n);

    // Update gauge with the *most recent* Fibonacci input
    LATEST_N_GAUGE.set(n as f64);

    // Acquire lock on daily_request_count
    let mut count = match daily_request_count.lock() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to acquire lock on daily_request_count: {}", e);
            // Track 500 (Internal Server Error)
            REQUEST_STATUS_COUNTER.with_label_values(&["500"]).inc();
            return HttpResponse::InternalServerError().body("Internal server error");
        }
    };

    // Example of returning a 400 if n is 0 (or invalid)
    if n == 0 {
        REQUEST_STATUS_COUNTER.with_label_values(&["400"]).inc();
        return HttpResponse::BadRequest().body("n must be > 0");
    }

    // Check if we've hit the limit
    if *count >= MAX_DAILY_REQUESTS {
        // Increment our "limit reached" counter
        REQUEST_LIMIT_REACHED_COUNTER.inc();
        // Track 429 (Too Many Requests)
        REQUEST_STATUS_COUNTER.with_label_values(&["429"]).inc();

        return HttpResponse::TooManyRequests().body(
            "Thank you for using our Fibonacci service! \
             You’ve reached the daily request limit. \
             Please come back tomorrow or contact us if you need additional requests.",
        );
    }

    // Important: increment the daily count
    *count += 1;

    // Increase the "active requests" gauge
    ACTIVE_REQUESTS.inc();

    // Start timing this request
    let timer = REQUEST_HISTOGRAM.start_timer();

    // Calculate Fibonacci
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

    // Track 200 (OK)
    REQUEST_STATUS_COUNTER.with_label_values(&["200"]).inc();
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
            REQUEST_STATUS_COUNTER.with_label_values(&["404"]).inc(); // track 404 here
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

    // Wrap the request count in an Actix Data<Mutex<u32>>
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
