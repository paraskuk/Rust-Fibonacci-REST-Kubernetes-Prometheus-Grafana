use actix_files as fs;
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    http::header,
    middleware::{DefaultHeaders, Logger},
    web, App, HttpResponse, HttpServer, Result as ActixResult,
};
use serde::{Deserialize, Serialize};
use std::sync::{LazyLock, Mutex};
use thiserror::Error;
use tokio_cron_scheduler::{Job, JobScheduler};

use fibonacci::fibonacci_iterative;
use log::{error, info};
use prometheus::{
    gather, register_counter, register_gauge, register_histogram, register_int_counter_vec,
    Encoder, Histogram, IntCounterVec, TextEncoder,
};

static STATIC_DIR: &str = "/usr/src/app/static";
static LOG4RS_CONFIG: &str = "/usr/src/app/log4rs.yaml";

const MAX_DAILY_REQUESTS: u32 = 1001;
const MAX_FIBONACCI_INPUT: u32 = 50; // Limit to prevent DoS

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Request limit reached")]
    RateLimitExceeded,
    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl actix_web::ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::InvalidInput(msg) => {
                REQUEST_STATUS_COUNTER.with_label_values(&["400"]).inc();
                HttpResponse::BadRequest().json(ErrorResponse { error: msg.clone() })
            }
            AppError::RateLimitExceeded => {
                REQUEST_LIMIT_REACHED_COUNTER.inc();
                REQUEST_STATUS_COUNTER.with_label_values(&["429"]).inc();
                HttpResponse::TooManyRequests().json(ErrorResponse {
                    error: "Thank you for using our Fibonacci service! \
                             You've reached the daily request limit. \
                             Please come back tomorrow or contact us if you need additional requests.".to_string(),
                })
            }
            AppError::InternalError(msg) => {
                REQUEST_STATUS_COUNTER.with_label_values(&["500"]).inc();
                HttpResponse::InternalServerError().json(ErrorResponse { error: msg.clone() })
            }
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Deserialize)]
struct FibonacciInput {
    n: u32,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Serialize)]
struct ReadinessResponse {
    status: String,
    checks: ReadinessChecks,
}

#[derive(Serialize)]
struct ReadinessChecks {
    daily_requests: String,
}

// Replace lazy_static with std::sync::LazyLock
static REQUEST_COUNTER: LazyLock<prometheus::Counter> =
    LazyLock::new(|| register_counter!("requests_total", "Total number of requests").unwrap());

static REQUEST_HISTOGRAM: LazyLock<prometheus::Histogram> = LazyLock::new(|| {
    register_histogram!("request_duration_seconds", "Request duration in seconds").unwrap()
});

static ACTIVE_REQUESTS: LazyLock<prometheus::Gauge> =
    LazyLock::new(|| register_gauge!("active_requests", "Number of active requests").unwrap());

static RESPONSE_SIZE_HISTOGRAM: LazyLock<prometheus::Histogram> =
    LazyLock::new(|| register_histogram!("response_size_bytes", "Response size in bytes").unwrap());

static REQUEST_LIMIT_REACHED_COUNTER: LazyLock<prometheus::Counter> = LazyLock::new(|| {
    register_counter!(
        "request_limit_reached_total",
        "Number of times the service returned TooManyRequests"
    )
    .unwrap()
});

static REQUEST_STATUS_COUNTER: LazyLock<IntCounterVec> = LazyLock::new(|| {
    register_int_counter_vec!(
        "request_status_codes_total",
        "Number of requests by HTTP status code",
        &["code"]
    )
    .unwrap()
});

static FIBONACCI_N_HISTOGRAM: LazyLock<Histogram> = LazyLock::new(|| {
    register_histogram!(
        "fibonacci_n_distribution",
        "Distribution of requested Fibonacci inputs (n)"
    )
    .unwrap()
});

// Health check endpoint
async fn health() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }))
}

// Readiness check endpoint
async fn ready(daily_request_count: web::Data<Mutex<u32>>) -> ActixResult<HttpResponse> {
    let count = daily_request_count
        .lock()
        .map_err(|e| AppError::InternalError(format!("Lock error: {}", e)))?;

    let status = if *count < MAX_DAILY_REQUESTS {
        "ready"
    } else {
        "not_ready"
    };

    Ok(HttpResponse::Ok().json(ReadinessResponse {
        status: status.to_string(),
        checks: ReadinessChecks {
            daily_requests: format!("{}/{}", count, MAX_DAILY_REQUESTS),
        },
    }))
}

// Handler for calculating Fibonacci with improved error handling
async fn calculate_fibonacci(
    data: web::Query<FibonacciInput>,
    daily_request_count: web::Data<Mutex<u32>>,
) -> ActixResult<HttpResponse> {
    let n = data.n;
    info!("Received request to calculate Fibonacci for n = {}", n);

    // Observe the distribution of 'n'
    FIBONACCI_N_HISTOGRAM.observe(n as f64);

    // Validate input
    if n == 0 {
        return Err(
            AppError::InvalidInput("Input value must be greater than 0".to_string()).into(),
        );
    }

    if n > MAX_FIBONACCI_INPUT {
        return Err(AppError::InvalidInput(format!(
            "Input value must be <= {} to prevent resource exhaustion",
            MAX_FIBONACCI_INPUT
        ))
        .into());
    }

    // Check rate limit and increment counter in a separate scope to drop the lock before await
    {
        let mut count = daily_request_count
            .lock()
            .map_err(|e| AppError::InternalError(format!("Failed to acquire lock: {}", e)))?;

        // Check if we've hit the daily limit
        if *count >= MAX_DAILY_REQUESTS {
            return Err(AppError::RateLimitExceeded.into());
        }

        // Increment the daily count
        *count += 1;
    } // Lock is dropped here

    // Increase the "active requests" gauge
    ACTIVE_REQUESTS.inc();

    // Time this request
    let timer = REQUEST_HISTOGRAM.start_timer();

    // Calculate Fibonacci using spawn_blocking to prevent blocking the async runtime
    let result = tokio::task::spawn_blocking(move || fibonacci_iterative(n))
        .await
        .map_err(|e| AppError::InternalError(format!("Calculation error: {}", e)))?;

    info!("Calculated Fibonacci for n = {}: {}", n, result);

    // Count this request in Prometheus metrics
    REQUEST_COUNTER.inc();

    // Stop the timer
    timer.observe_duration();

    // Decrement "active requests"
    ACTIVE_REQUESTS.dec();

    // Observe response size
    let response_size = serde_json::to_string(&result)
        .map_err(|e| AppError::InternalError(format!("Serialization error: {}", e)))?
        .len() as f64;
    RESPONSE_SIZE_HISTOGRAM.observe(response_size);

    // Track 200 (OK)
    REQUEST_STATUS_COUNTER.with_label_values(&["200"]).inc();
    Ok(HttpResponse::Ok().json(result))
}

// Handler for Prometheus metrics
async fn metrics() -> impl actix_web::Responder {
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
            REQUEST_STATUS_COUNTER.with_label_values(&["404"]).inc();
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

    // Shared state for daily requests
    let daily_request_count = web::Data::new(Mutex::new(0u32));

    // Reset count daily at midnight
    let scheduler = JobScheduler::new().await.unwrap();
    let daily_request_count_clone = daily_request_count.clone();

    scheduler
        .add(
            Job::new_async("0 0 * * * *", move |_uuid, _l| {
                let daily_request_count_clone = daily_request_count_clone.clone();
                Box::pin(async move {
                    let mut count = daily_request_count_clone.lock().unwrap();
                    *count = 0;
                    info!("Daily request count reset to 0");
                })
            })
            .unwrap(),
        )
        .await
        .unwrap();

    scheduler.start().await.unwrap();

    HttpServer::new(move || {
        // Configure CORS
        let cors = actix_cors::Cors::default()
            .allowed_origin("http://localhost:8080")
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::ACCEPT,
                header::CONTENT_TYPE,
            ])
            .max_age(3600);

        App::new()
            .app_data(daily_request_count.clone())
            .app_data(web::Data::new(web::PayloadConfig::new(1024 * 1024))) // 1MB max payload
            .wrap(Logger::default())
            .wrap(cors)
            // Add security headers
            .wrap(
                DefaultHeaders::new()
                    .add((header::X_FRAME_OPTIONS, "DENY"))
                    .add((header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
                    .add(("X-XSS-Protection", "1; mode=block"))
                    .add(("Content-Security-Policy",
                        "default-src 'self'; \
                         style-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net https://cdnjs.cloudflare.com https://fonts.googleapis.com; \
                         script-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net; \
                         font-src 'self' https://fonts.gstatic.com https://cdnjs.cloudflare.com; \
                         img-src 'self' data:"))
                    .add((
                        header::STRICT_TRANSPORT_SECURITY,
                        "max-age=31536000; includeSubDomains",
                    )),
            )
            // Health and readiness endpoints
            .route("/health", web::get().to(health))
            .route("/ready", web::get().to(ready))
            // Business endpoints
            .route("/fibonacci", web::get().to(calculate_fibonacci))
            .route("/metrics", web::get().to(metrics))
            .service(
                fs::Files::new("/", STATIC_DIR)
                    .index_file("index.html")
                    .default_handler(default_handler),
            )
    })
    .bind("0.0.0.0:8080")?
    .keep_alive(std::time::Duration::from_secs(60))
    .client_request_timeout(std::time::Duration::from_secs(30))
    .run()
    .await
}
