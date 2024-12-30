provider "kubernetes" {
  config_path = "~/.kube/config"
}

# Fibonacci Namespace
resource "kubernetes_namespace" "fibonacci" {
  metadata {
    name = "fibonacci"
  }
}

# Fibonacci Deployment
resource "kubernetes_deployment" "fibonacci" {
  metadata {
    name      = "fibonacci-deployment"
    namespace = kubernetes_namespace.fibonacci.metadata[0].name
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
        image_pull_secrets {
          name = "regcred"
        }

        container {
          image = "paraskevas68/fibonacci_rust:v36"
          name  = "fibonacci"

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
            initial_delay_seconds = 10
            period_seconds        = 10
          }

          readiness_probe {
            http_get {
              path = "/fibonacci?n=1"
              port = 8080
            }
            initial_delay_seconds = 10
            period_seconds        = 10
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
            name = "fibonacci-log4rs"
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
    namespace = kubernetes_namespace.fibonacci.metadata[0].name
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
          image = "grafana/grafana:latest"
          name  = "grafana"

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
          image = "otel/opentelemetry-collector:latest"
          name  = "otel-collector"

          command = ["/otelcol", "--config=/conf/otel-collector-config.yaml"]

          volume_mount {
            name       = "otel-collector-config-vol"
            mount_path = "/conf"
          }
        }

        volume {
          name = "otel-collector-config-vol"
          config_map {
            name = "otel-collector-config"
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
          image = "prom/prometheus:latest"
          name  = "prometheus"

          port {
            container_port = 9090
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