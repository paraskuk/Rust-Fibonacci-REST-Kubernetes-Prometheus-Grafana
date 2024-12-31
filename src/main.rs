use actix_files as fs;
use actix_web::{web, App, HttpServer, Responder, HttpResponse, middleware::Logger};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use serde::Deserialize;
use fibonacci::fibonacci_iterative;
use log::{info, error};
use log4rs;
use prometheus::{Encoder, TextEncoder, register_counter, register_histogram, register_gauge, gather};
use std::sync::{Arc, Mutex};
use tokio_cron_scheduler::{Job, JobScheduler};

static STATIC_DIR: &str = "/usr/src/app/static";
static LOG4RS_CONFIG: &str = "/usr/src/app/log4rs.yaml";
const MAX_DAILY_REQUESTS: u32 = 1000; // Define the maximum daily requests as a constant

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

async fn calculate_fibonacci(data: web::Query<FibonacciInput>, daily_request_count: web::Data<Arc<Mutex<u32>>>) -> impl Responder {
    let n = data.n;
    if n < 0 {
        error!("Received negative number for Fibonacci calculation: {}", n);
        return HttpResponse::BadRequest().body("Negative numbers are not allowed");
    }
    info!("Received request to calculate Fibonacci for n = {}", n);

    // Check daily request limit
    let mut count = daily_request_count.lock().unwrap();
    if *count >= MAX_DAILY_REQUESTS {
        return HttpResponse::TooManyRequests().body("Daily request limit reached for this POC site reached. Please visit tomorrow as I dont want to have high cloud costs!");
    }
    *count += 1;

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

    // Shared state for daily request count
    let daily_request_count = Arc::new(Mutex::new(0));

    // Scheduler to reset the daily request count at midnight
    let scheduler = JobScheduler::new().await.unwrap();
    let daily_request_count_clone = Arc::clone(&daily_request_count);
    scheduler.add(Job::new_async("0 0 * * * *", move |_uuid, _l| {
        let daily_request_count_clone = Arc::clone(&daily_request_count_clone);
        Box::pin(async move {
            let mut count = daily_request_count_clone.lock().unwrap();
            *count = 0;
            println!("Daily request count reset to 0");
        })
    }).unwrap()).await.unwrap();
    scheduler.start().await.unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Arc::clone(&daily_request_count)))
            .wrap(Logger::default())
            .route("/fibonacci", web::get().to(calculate_fibonacci))
            .route("/metrics", web::get().to(metrics))
            .service(
                fs::Files::new("/", STATIC_DIR)
                    .index_file("index.html")
                    .default_handler(default_handler)
            )
    })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}