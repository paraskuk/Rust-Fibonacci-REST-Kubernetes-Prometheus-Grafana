# Terraform RBAC Configuration Template

This file provides production-ready Terraform configurations for implementing authentication and authorization in the Fibonacci Kubernetes cluster.

## Complete Terraform RBAC Implementation

Add the following to your `main.tf` file:

### 1. Namespace Resources

```hcl
# Create dedicated namespaces
resource "kubernetes_namespace" "fibonacci_app" {
  metadata {
    name = "fibonacci-app"
    labels = {
      "security-tier" = "application"
      "monitoring"    = "enabled"
      "backup"        = "required"
    }
    annotations = {
      "description" = "Namespace for Fibonacci application"
    }
  }
}

resource "kubernetes_namespace" "monitoring" {
  metadata {
    name = "monitoring"
    labels = {
      "security-tier" = "monitoring"
      "monitoring"    = "enabled"
      "backup"        = "required"
    }
    annotations = {
      "description" = "Namespace for monitoring infrastructure"
    }
  }
}

resource "kubernetes_namespace" "auth_system" {
  metadata {
    name = "auth-system"
    labels = {
      "security-tier" = "authentication"
      "monitoring"    = "enabled"
      "backup"        = "critical"
    }
    annotations = {
      "description" = "Namespace for authentication and authorization services"
    }
  }
}
```

### 2. Service Accounts

```hcl
# Fibonacci application service account
resource "kubernetes_service_account" "fibonacci_sa" {
  metadata {
    name      = "fibonacci-service-account"
    namespace = kubernetes_namespace.fibonacci_app.metadata[0].name
    labels = {
      app           = "fibonacci"
      component     = "backend"
      security-tier = "application"
    }
    annotations = {
      "description" = "Service account for Fibonacci application pods"
    }
  }
  
  automount_service_account_token = true
}

# Prometheus service account
resource "kubernetes_service_account" "prometheus_sa" {
  metadata {
    name      = "prometheus-service-account"
    namespace = kubernetes_namespace.monitoring.metadata[0].name
    labels = {
      app           = "prometheus"
      component     = "monitoring"
      security-tier = "monitoring"
    }
    annotations = {
      "description" = "Service account for Prometheus monitoring"
    }
  }
  
  automount_service_account_token = true
}

# OpenTelemetry Collector service account
resource "kubernetes_service_account" "otel_collector_sa" {
  metadata {
    name      = "otel-collector-service-account"
    namespace = kubernetes_namespace.monitoring.metadata[0].name
    labels = {
      app           = "otel-collector"
      component     = "telemetry"
      security-tier = "monitoring"
    }
    annotations = {
      "description" = "Service account for OpenTelemetry Collector"
    }
  }
  
  automount_service_account_token = true
}

# Grafana service account
resource "kubernetes_service_account" "grafana_sa" {
  metadata {
    name      = "grafana-service-account"
    namespace = kubernetes_namespace.monitoring.metadata[0].name
    labels = {
      app           = "grafana"
      component     = "dashboard"
      security-tier = "monitoring"
    }
    annotations = {
      "description" = "Service account for Grafana dashboard"
    }
  }
  
  automount_service_account_token = true
}
```

### 3. Cluster Roles

```hcl
# Admin cluster role with full permissions
resource "kubernetes_cluster_role" "fibonacci_cluster_admin" {
  metadata {
    name = "fibonacci-cluster-admin"
    labels = {
      security-tier = "admin"
      rbac-scope   = "cluster"
    }
    annotations = {
      "description" = "Full cluster administration role for Fibonacci project"
    }
  }

  rule {
    api_groups = ["*"]
    resources  = ["*"]
    verbs      = ["*"]
  }

  rule {
    non_resource_urls = ["*"]
    verbs             = ["*"]
  }
}

# Reader cluster role with read-only permissions
resource "kubernetes_cluster_role" "fibonacci_cluster_reader" {
  metadata {
    name = "fibonacci-cluster-reader"
    labels = {
      security-tier = "reader"
      rbac-scope   = "cluster"
    }
    annotations = {
      "description" = "Read-only cluster access for Fibonacci project"
    }
  }

  # Pod and basic resource read access
  rule {
    api_groups = [""]
    resources  = ["pods", "services", "configmaps", "endpoints", "persistentvolumeclaims"]
    verbs      = ["get", "list", "watch"]
  }

  # Application resource read access
  rule {
    api_groups = ["apps"]
    resources  = ["deployments", "replicasets", "daemonsets", "statefulsets"]
    verbs      = ["get", "list", "watch"]
  }

  # Networking read access
  rule {
    api_groups = ["networking.k8s.io"]
    resources  = ["networkpolicies", "ingresses"]
    verbs      = ["get", "list", "watch"]
  }

  # Monitoring resources read access
  rule {
    api_groups = ["monitoring.coreos.com"]
    resources  = ["servicemonitors", "prometheusrules", "prometheuses", "alertmanagers"]
    verbs      = ["get", "list", "watch"]
  }

  # Log access
  rule {
    api_groups = [""]
    resources  = ["pods/log"]
    verbs      = ["get", "list"]
  }

  # Metrics access
  rule {
    api_groups = ["metrics.k8s.io"]
    resources  = ["pods", "nodes"]
    verbs      = ["get", "list"]
  }
}

# Monitoring cluster role for monitoring services
resource "kubernetes_cluster_role" "fibonacci_monitoring" {
  metadata {
    name = "fibonacci-monitoring"
    labels = {
      security-tier = "monitoring"
      rbac-scope   = "cluster"
    }
    annotations = {
      "description" = "Monitoring services cluster role"
    }
  }

  # Access to metrics and monitoring endpoints
  rule {
    api_groups = [""]
    resources  = ["nodes", "nodes/proxy", "nodes/metrics", "services", "endpoints", "pods"]
    verbs      = ["get", "list", "watch"]
  }

  rule {
    api_groups = ["extensions"]
    resources  = ["ingresses"]
    verbs      = ["get", "list", "watch"]
  }

  rule {
    non_resource_urls = ["/metrics", "/metrics/*"]
    verbs             = ["get"]
  }
}
```

### 4. Namespace-Specific Roles

```hcl
# Application manager role for fibonacci-app namespace
resource "kubernetes_role" "fibonacci_app_manager" {
  metadata {
    namespace = kubernetes_namespace.fibonacci_app.metadata[0].name
    name      = "fibonacci-app-manager"
    labels = {
      security-tier = "application"
      rbac-scope   = "namespace"
    }
    annotations = {
      "description" = "Management role for Fibonacci application namespace"
    }
  }

  # Full access to application resources
  rule {
    api_groups = [""]
    resources  = ["pods", "services", "configmaps", "secrets", "persistentvolumeclaims", "serviceaccounts"]
    verbs      = ["get", "list", "watch", "create", "update", "patch", "delete"]
  }

  rule {
    api_groups = ["apps"]
    resources  = ["deployments", "replicasets", "daemonsets", "statefulsets"]
    verbs      = ["get", "list", "watch", "create", "update", "patch", "delete"]
  }

  rule {
    api_groups = ["networking.k8s.io"]
    resources  = ["networkpolicies"]
    verbs      = ["get", "list", "watch", "create", "update", "patch", "delete"]
  }

  rule {
    api_groups = ["rbac.authorization.k8s.io"]
    resources  = ["roles", "rolebindings"]
    verbs      = ["get", "list", "watch", "create", "update", "patch", "delete"]
  }
}

# Monitoring manager role for monitoring namespace
resource "kubernetes_role" "monitoring_manager" {
  metadata {
    namespace = kubernetes_namespace.monitoring.metadata[0].name
    name      = "monitoring-manager"
    labels = {
      security-tier = "monitoring"
      rbac-scope   = "namespace"
    }
    annotations = {
      "description" = "Management role for monitoring namespace"
    }
  }

  # Full access to monitoring resources
  rule {
    api_groups = [""]
    resources  = ["pods", "services", "configmaps", "secrets", "persistentvolumeclaims", "serviceaccounts"]
    verbs      = ["get", "list", "watch", "create", "update", "patch", "delete"]
  }

  rule {
    api_groups = ["apps"]
    resources  = ["deployments", "replicasets", "daemonsets", "statefulsets"]
    verbs      = ["get", "list", "watch", "create", "update", "patch", "delete"]
  }

  rule {
    api_groups = ["monitoring.coreos.com"]
    resources  = ["servicemonitors", "prometheusrules", "prometheuses", "alertmanagers"]
    verbs      = ["get", "list", "watch", "create", "update", "patch", "delete"]
  }
}

# Application reader role for fibonacci-app namespace
resource "kubernetes_role" "fibonacci_app_reader" {
  metadata {
    namespace = kubernetes_namespace.fibonacci_app.metadata[0].name
    name      = "fibonacci-app-reader"
    labels = {
      security-tier = "reader"
      rbac-scope   = "namespace"
    }
    annotations = {
      "description" = "Read-only role for Fibonacci application namespace"
    }
  }

  # Read-only access to application resources
  rule {
    api_groups = [""]
    resources  = ["pods", "services", "configmaps", "endpoints"]
    verbs      = ["get", "list", "watch"]
  }

  rule {
    api_groups = ["apps"]
    resources  = ["deployments", "replicasets"]
    verbs      = ["get", "list", "watch"]
  }

  rule {
    api_groups = [""]
    resources  = ["pods/log"]
    verbs      = ["get", "list"]
  }
}
```

### 5. Role Bindings

```hcl
# Cluster role bindings
resource "kubernetes_cluster_role_binding" "fibonacci_admin_binding" {
  metadata {
    name = "fibonacci-admin-binding"
    labels = {
      security-tier = "admin"
      rbac-scope   = "cluster"
    }
    annotations = {
      "description" = "Binds admin users to cluster admin role"
    }
  }

  role_ref {
    api_group = "rbac.authorization.k8s.io"
    kind      = "ClusterRole"
    name      = kubernetes_cluster_role.fibonacci_cluster_admin.metadata[0].name
  }

  # Bind to admin users (replace with actual usernames)
  subject {
    kind      = "User"
    name      = "admin"
    api_group = "rbac.authorization.k8s.io"
  }

  # Bind to fibonacci service account for admin operations
  subject {
    kind      = "ServiceAccount"
    name      = kubernetes_service_account.fibonacci_sa.metadata[0].name
    namespace = kubernetes_service_account.fibonacci_sa.metadata[0].namespace
  }

  # Bind to admin group
  subject {
    kind      = "Group"
    name      = "system:masters"
    api_group = "rbac.authorization.k8s.io"
  }
}

resource "kubernetes_cluster_role_binding" "fibonacci_reader_binding" {
  metadata {
    name = "fibonacci-reader-binding"
    labels = {
      security-tier = "reader"
      rbac-scope   = "cluster"
    }
    annotations = {
      "description" = "Binds reader users to cluster reader role"
    }
  }

  role_ref {
    api_group = "rbac.authorization.k8s.io"
    kind      = "ClusterRole"
    name      = kubernetes_cluster_role.fibonacci_cluster_reader.metadata[0].name
  }

  # Bind to reader users
  subject {
    kind      = "User"
    name      = "reader"
    api_group = "rbac.authorization.k8s.io"
  }

  # Bind to readers group
  subject {
    kind      = "Group"
    name      = "readers"
    api_group = "rbac.authorization.k8s.io"
  }
}

resource "kubernetes_cluster_role_binding" "fibonacci_monitoring_binding" {
  metadata {
    name = "fibonacci-monitoring-binding"
    labels = {
      security-tier = "monitoring"
      rbac-scope   = "cluster"
    }
    annotations = {
      "description" = "Binds monitoring services to monitoring cluster role"
    }
  }

  role_ref {
    api_group = "rbac.authorization.k8s.io"
    kind      = "ClusterRole"
    name      = kubernetes_cluster_role.fibonacci_monitoring.metadata[0].name
  }

  subject {
    kind      = "ServiceAccount"
    name      = kubernetes_service_account.prometheus_sa.metadata[0].name
    namespace = kubernetes_service_account.prometheus_sa.metadata[0].namespace
  }

  subject {
    kind      = "ServiceAccount"
    name      = kubernetes_service_account.otel_collector_sa.metadata[0].name
    namespace = kubernetes_service_account.otel_collector_sa.metadata[0].namespace
  }
}

# Namespace role bindings
resource "kubernetes_role_binding" "fibonacci_app_manager_binding" {
  metadata {
    name      = "fibonacci-app-manager-binding"
    namespace = kubernetes_namespace.fibonacci_app.metadata[0].name
    labels = {
      security-tier = "application"
      rbac-scope   = "namespace"
    }
    annotations = {
      "description" = "Binds application managers to app manager role"
    }
  }

  role_ref {
    api_group = "rbac.authorization.k8s.io"
    kind      = "Role"
    name      = kubernetes_role.fibonacci_app_manager.metadata[0].name
  }

  subject {
    kind      = "ServiceAccount"
    name      = kubernetes_service_account.fibonacci_sa.metadata[0].name
    namespace = kubernetes_service_account.fibonacci_sa.metadata[0].namespace
  }

  # Add app managers group
  subject {
    kind      = "Group"
    name      = "fibonacci-app-managers"
    api_group = "rbac.authorization.k8s.io"
  }
}

resource "kubernetes_role_binding" "monitoring_manager_binding" {
  metadata {
    name      = "monitoring-manager-binding"
    namespace = kubernetes_namespace.monitoring.metadata[0].name
    labels = {
      security-tier = "monitoring"
      rbac-scope   = "namespace"
    }
    annotations = {
      "description" = "Binds monitoring managers to monitoring manager role"
    }
  }

  role_ref {
    api_group = "rbac.authorization.k8s.io"
    kind      = "Role"
    name      = kubernetes_role.monitoring_manager.metadata[0].name
  }

  subject {
    kind      = "ServiceAccount"
    name      = kubernetes_service_account.prometheus_sa.metadata[0].name
    namespace = kubernetes_service_account.prometheus_sa.metadata[0].namespace
  }

  subject {
    kind      = "ServiceAccount"
    name      = kubernetes_service_account.otel_collector_sa.metadata[0].name
    namespace = kubernetes_service_account.otel_collector_sa.metadata[0].namespace
  }

  subject {
    kind      = "ServiceAccount"
    name      = kubernetes_service_account.grafana_sa.metadata[0].name
    namespace = kubernetes_service_account.grafana_sa.metadata[0].namespace
  }
}

resource "kubernetes_role_binding" "fibonacci_app_reader_binding" {
  metadata {
    name      = "fibonacci-app-reader-binding"
    namespace = kubernetes_namespace.fibonacci_app.metadata[0].name
    labels = {
      security-tier = "reader"
      rbac-scope   = "namespace"
    }
    annotations = {
      "description" = "Binds app readers to app reader role"
    }
  }

  role_ref {
    api_group = "rbac.authorization.k8s.io"
    kind      = "Role"
    name      = kubernetes_role.fibonacci_app_reader.metadata[0].name
  }

  # Bind to readers group
  subject {
    kind      = "Group"
    name      = "fibonacci-app-readers"
    api_group = "rbac.authorization.k8s.io"
  }
}
```

### 6. Secrets for Authentication

```hcl
# JWT secret for application authentication
resource "kubernetes_secret" "fibonacci_jwt_secret" {
  metadata {
    name      = "fibonacci-jwt-secret"
    namespace = kubernetes_namespace.fibonacci_app.metadata[0].name
    labels = {
      app           = "fibonacci"
      security-tier = "application"
      secret-type   = "authentication"
    }
    annotations = {
      "description" = "JWT signing secret for Fibonacci application"
    }
  }

  type = "Opaque"

  data = {
    jwt-secret = base64encode(var.jwt_secret_key)
  }
}

# OIDC configuration secret
resource "kubernetes_secret" "oidc_config" {
  metadata {
    name      = "oidc-config"
    namespace = kubernetes_namespace.auth_system.metadata[0].name
    labels = {
      security-tier = "authentication"
      secret-type   = "oidc"
    }
    annotations = {
      "description" = "OIDC provider configuration"
    }
  }

  type = "Opaque"

  data = {
    oidc-issuer-url = base64encode(var.oidc_issuer_url)
    oidc-client-id  = base64encode(var.oidc_client_id)
    oidc-client-secret = base64encode(var.oidc_client_secret)
  }
}
```

### 7. Network Policies

```hcl
# Network policy for Fibonacci application
resource "kubernetes_network_policy" "fibonacci_app_netpol" {
  metadata {
    name      = "fibonacci-app-network-policy"
    namespace = kubernetes_namespace.fibonacci_app.metadata[0].name
    labels = {
      app           = "fibonacci"
      security-tier = "application"
    }
    annotations = {
      "description" = "Network policy for Fibonacci application"
    }
  }

  spec {
    pod_selector {
      match_labels = {
        app = "fibonacci"
      }
    }

    policy_types = ["Ingress", "Egress"]

    # Allow ingress from monitoring namespace and anywhere for service access
    ingress {
      from {
        namespace_selector {
          match_labels = {
            security-tier = "monitoring"
          }
        }
      }
      ports {
        port     = "8080"
        protocol = "TCP"
      }
    }

    ingress {
      from = []  # Allow from anywhere for external access
      ports {
        port     = "8080"
        protocol = "TCP"
      }
    }

    # Allow egress to monitoring namespace and DNS
    egress {
      to {
        namespace_selector {
          match_labels = {
            security-tier = "monitoring"
          }
        }
      }
      ports {
        port     = "8889"
        protocol = "TCP"
      }
    }

    # Allow DNS resolution
    egress {
      to = []
      ports {
        port     = "53"
        protocol = "UDP"
      }
    }

    # Allow HTTPS for external API calls
    egress {
      to = []
      ports {
        port     = "443"
        protocol = "TCP"
      }
    }
  }
}

# Network policy for monitoring namespace
resource "kubernetes_network_policy" "monitoring_netpol" {
  metadata {
    name      = "monitoring-network-policy"
    namespace = kubernetes_namespace.monitoring.metadata[0].name
    labels = {
      security-tier = "monitoring"
    }
    annotations = {
      "description" = "Network policy for monitoring namespace"
    }
  }

  spec {
    pod_selector = {}  # Apply to all pods in namespace

    policy_types = ["Ingress", "Egress"]

    # Allow ingress from application and monitoring namespaces
    ingress {
      from {
        namespace_selector {
          match_labels = {
            security-tier = "application"
          }
        }
      }
    }

    ingress {
      from {
        namespace_selector {
          match_labels = {
            security-tier = "monitoring"
          }
        }
      }
    }

    # Allow external access to Grafana and Prometheus
    ingress {
      from = []
      ports {
        port     = "3000"  # Grafana
        protocol = "TCP"
      }
    }

    ingress {
      from = []
      ports {
        port     = "9090"  # Prometheus
        protocol = "TCP"
      }
    }

    # Allow all egress for monitoring tools
    egress {
      to = []
    }
  }
}
```

### 8. Variables

Add these variables to your `variables.tf`:

```hcl
variable "jwt_secret_key" {
  description = "Secret key for JWT token signing"
  type        = string
  sensitive   = true
  default     = ""
}

variable "oidc_issuer_url" {
  description = "OIDC provider issuer URL"
  type        = string
  default     = ""
}

variable "oidc_client_id" {
  description = "OIDC client ID"
  type        = string
  default     = ""
}

variable "oidc_client_secret" {
  description = "OIDC client secret"
  type        = string
  sensitive   = true
  default     = ""
}

variable "enable_rbac" {
  description = "Enable RBAC configuration"
  type        = bool
  default     = true
}

variable "enable_network_policies" {
  description = "Enable network policies"
  type        = bool
  default     = true
}
```

## Deployment Instructions

1. **Update your terraform configuration:**
   ```bash
   terraform plan -var="jwt_secret_key=your-secret-key" -var="enable_rbac=true"
   terraform apply -var="jwt_secret_key=your-secret-key" -var="enable_rbac=true"
   ```

2. **Verify RBAC setup:**
   ```bash
   kubectl get clusterroles | grep fibonacci
   kubectl get clusterrolebindings | grep fibonacci
   kubectl get roles -A | grep fibonacci
   kubectl get rolebindings -A | grep fibonacci
   ```

3. **Test permissions:**
   ```bash
   kubectl auth can-i create deployments --as=admin
   kubectl auth can-i create deployments --as=reader  # Should be "no"
   ```

This Terraform configuration provides a complete, production-ready RBAC setup for the Fibonacci Kubernetes cluster with proper separation of concerns and security best practices.