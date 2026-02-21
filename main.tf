provider "kubernetes" {
  config_path = "~/.kube/config"
}

# Fibonacci ConfigMap for log4rs
resource "kubernetes_config_map" "fibonacci_log4rs" {
  metadata {
    name      = "fibonacci-log4rs"
    namespace = "default"
  }

  data = {
    "log4rs.yaml" = <<-EOT
appenders:
  file:
    kind: file
    path: "/var/log/fibonacci.log"
    encoder:
      pattern: "{d} - {l} - {m}{n}"
root:
  level: debug
  appenders:
    - file
EOT
  }
}

# Prometheus ConfigMap
resource "kubernetes_config_map" "prometheus_config" {
  metadata {
    name      = "prometheus-config"
    namespace = "default"
  }

  data = {
    "prometheus.yml" = <<-EOT
global:
  scrape_interval: 15s
scrape_configs:
  - job_name: 'otel-collector'
    static_configs:
      - targets: ['otel-collector:8889']
  - job_name: 'fibonacci'
    static_configs:
      - targets: ['fibonacci-service:8080']
EOT
  }
}

# OpenTelemetry Collector ConfigMap
resource "kubernetes_config_map" "otel_collector_config" {
  metadata {
    name      = "otel-collector-config"
    namespace = "default"
  }

  data = {
    "otel-collector-config.yaml" = <<-EOT
receivers:
  otlp:
    protocols:
      grpc:
      http:
processors:
  batch:
exporters:
  prometheus:
    endpoint: "0.0.0.0:8889"
service:
  pipelines:
    metrics:
      receivers: [otlp]
      processors: [batch]
      exporters: [prometheus]
EOT
  }
}

# Fibonacci Deployment
resource "kubernetes_deployment" "fibonacci" {
  metadata {
    name      = "fibonacci-deployment"
    namespace = "default"
  }

  spec {
    replicas = 1

    selector {
      match_labels = {
        app = "fibonacci"
      }
    }

    template {
      metadata {
        labels = {
          app = "fibonacci"
        }
      }

      spec {

        container {
          name  = "fibonacci"
          image = "${var.docker_username}/fibonacci_rust:${var.docker_image_tag}"

          resources {
            limits = {
              cpu    = "100m"
              memory = "128Mi"
            }
            requests = {
              cpu    = "100m"
              memory = "128Mi"
            }
          }

          port {
            container_port = 8080
          }

          liveness_probe {
            http_get {
              path = "/fibonacci?n=1"
              port = 8080
            }
            initial_delay_seconds = 60
            period_seconds        = 10
            timeout_seconds       = 10
          }

          readiness_probe {
            http_get {
              path = "/fibonacci?n=1"
              port = 8080
            }
            initial_delay_seconds = 60
            period_seconds        = 10
            timeout_seconds       = 10
          }

          volume_mount {
            name       = "log4rs-config"
            mount_path = "/usr/src/app/log4rs.yaml"
            sub_path   = "log4rs.yaml"
          }

          volume_mount {
            name       = "log-volume"
            mount_path = "/var/log"
          }
        }

        volume {
          name = "log4rs-config"
          config_map {
            name = kubernetes_config_map.fibonacci_log4rs.metadata[0].name
            items {
              key  = "log4rs.yaml"
              path = "log4rs.yaml"
            }
          }
        }

        volume {
          name = "log-volume"
          empty_dir {}
        }
      }
    }
  }
}

# Fibonacci Service
resource "kubernetes_service" "fibonacci" {
  metadata {
    name      = "fibonacci-service"
    namespace = "default"
  }

  spec {
    selector = {
      app = "fibonacci"
    }

    type = "NodePort"

    port {
      port        = 8080
      target_port = 8080
      node_port   = 30000
    }
  }
}

# Grafana Deployment
resource "kubernetes_deployment" "grafana" {
  metadata {
    name      = "grafana"
    namespace = "default"
  }

  spec {
    replicas = 1

    selector {
      match_labels = {
        app = "grafana"
      }
    }

    template {
      metadata {
        labels = {
          app = "grafana"
        }
      }

      spec {
        container {
          name  = "grafana"
          image = "grafana/grafana:latest"

          port {
            container_port = 3000
          }
        }
      }
    }
  }
}

# Grafana Service
resource "kubernetes_service" "grafana" {
  metadata {
    name      = "grafana"
    namespace = "default"
  }

  spec {
    selector = {
      app = "grafana"
    }

    port {
      port = 3000
      name = "web"
    }
  }
}

# OpenTelemetry Collector Deployment
resource "kubernetes_deployment" "otel_collector" {
  metadata {
    name      = "otel-collector"
    namespace = "default"
  }

  spec {
    replicas = 1

    selector {
      match_labels = {
        app = "otel-collector"
      }
    }

    template {
      metadata {
        labels = {
          app = "otel-collector"
        }
      }

      spec {
        container {
          name  = "otel-collector"
          image = "otel/opentelemetry-collector:latest"

          command = ["/otelcol", "--config=/conf/otel-collector-config.yaml"]

          volume_mount {
            name       = "otel-collector-config-vol"
            mount_path = "/conf"
          }
        }

        volume {
          name = "otel-collector-config-vol"
          config_map {
            name = kubernetes_config_map.otel_collector_config.metadata[0].name
          }
        }
      }
    }
  }
}

# OpenTelemetry Collector Service
resource "kubernetes_service" "otel_collector" {
  metadata {
    name      = "otel-collector"
    namespace = "default"
  }

  spec {
    selector = {
      app = "otel-collector"
    }

    port {
      port = 4317
      name = "otlp-grpc"
    }

    port {
      port = 4318
      name = "otlp-http"
    }

    port {
      port = 8889
      name = "prometheus"
    }
  }
}

# Prometheus Deployment
resource "kubernetes_deployment" "prometheus" {
  metadata {
    name      = "prometheus"
    namespace = "default"
  }

  spec {
    replicas = 1

    selector {
      match_labels = {
        app = "prometheus"
      }
    }

    template {
      metadata {
        labels = {
          app = "prometheus"
        }
      }

      spec {
        container {
          name  = "prometheus"
          image = "prom/prometheus:latest"

          args = ["--config.file=/etc/prometheus/prometheus.yml"]

          port {
            container_port = 9090
          }

          volume_mount {
            name       = "prometheus-config-vol"
            mount_path = "/etc/prometheus"
          }
        }

        volume {
          name = "prometheus-config-vol"
          config_map {
            name = kubernetes_config_map.prometheus_config.metadata[0].name
          }
        }
      }
    }
  }
}

# Prometheus Service
resource "kubernetes_service" "prometheus" {
  metadata {
    name      = "prometheus"
    namespace = "default"
  }

  spec {
    selector = {
      app = "prometheus"
    }

    port {
      port = 9090
      name = "web"
    }
  }
}