# Kubernetes Authentication and Authorization Implementation Guide

This document provides comprehensive instructions for implementing authentication and authorization in the Fibonacci Kubernetes cluster. This guide is designed for LLM programs to follow step-by-step to secure the cluster with proper RBAC (Role-Based Access Control) policies.

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Authentication Setup](#authentication-setup)
4. [Authorization with RBAC](#authorization-with-rbac)
5. [Role Definitions](#role-definitions)
6. [Implementation Steps](#implementation-steps)
7. [Security Considerations](#security-considerations)
8. [Testing and Verification](#testing-and-verification)
9. [Troubleshooting](#troubleshooting)

## Overview

The current Fibonacci application runs in the `default` namespace without proper authentication and authorization controls. This guide will implement:

- **Authentication**: Verify the identity of users and services
- **Authorization**: Control what authenticated users and services can do
- **RBAC Policies**: Define granular permissions for different roles
- **Service Accounts**: Secure pod-to-pod communication
- **Network Policies**: Control traffic flow between services

### Current Architecture Analysis

The existing deployment includes:
- Fibonacci web application (port 8080)
- Prometheus monitoring (port 9090)
- OpenTelemetry Collector (port 8889)
- Grafana dashboard
- All services running in `default` namespace

## Prerequisites

Before implementing authentication and authorization, ensure you have:

- Kubernetes cluster with RBAC enabled (default in most modern clusters)
- `kubectl` configured with cluster-admin privileges
- Understanding of Kubernetes RBAC concepts
- Terraform and Helm installed (for deployment updates)

## Authentication Setup

### 1. Create Dedicated Namespaces

First, segregate services into appropriate namespaces for better security isolation:

```yaml
# Create namespaces
apiVersion: v1
kind: Namespace
metadata:
  name: fibonacci-app
  labels:
    security-tier: application
---
apiVersion: v1
kind: Namespace
metadata:
  name: monitoring
  labels:
    security-tier: monitoring
---
apiVersion: v1
kind: Namespace
metadata:
  name: auth-system
  labels:
    security-tier: authentication
```

### 2. Service Account Creation

Create service accounts for different components with minimal required permissions:

```yaml
# Fibonacci application service account
apiVersion: v1
kind: ServiceAccount
metadata:
  name: fibonacci-service-account
  namespace: fibonacci-app
  labels:
    app: fibonacci
    security.tier: application
---
# Prometheus service account
apiVersion: v1
kind: ServiceAccount
metadata:
  name: prometheus-service-account
  namespace: monitoring
  labels:
    app: prometheus
    security.tier: monitoring
---
# OpenTelemetry Collector service account
apiVersion: v1
kind: ServiceAccount
metadata:
  name: otel-collector-service-account
  namespace: monitoring
  labels:
    app: otel-collector
    security.tier: monitoring
```

### 3. User Authentication Setup

For user authentication, implement one of the following methods:

#### Option A: Certificate-based Authentication

```bash
# Generate client certificate for admin user
openssl genrsa -out admin.key 2048
openssl req -new -key admin.key -out admin.csr -subj "/CN=admin/O=system:masters"
openssl x509 -req -in admin.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out admin.crt -days 365

# Generate client certificate for reader user
openssl genrsa -out reader.key 2048
openssl req -new -key reader.key -out reader.csr -subj "/CN=reader/O=readers"
openssl x509 -req -in reader.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out reader.crt -days 365
```

#### Option B: OIDC Integration (Recommended for Production)

```yaml
# OIDC configuration for kube-apiserver
apiVersion: v1
kind: ConfigMap
metadata:
  name: oidc-config
  namespace: kube-system
data:
  oidc-issuer-url: "https://your-oidc-provider.com"
  oidc-client-id: "kubernetes"
  oidc-username-claim: "email"
  oidc-groups-claim: "groups"
```

## Authorization with RBAC

### 1. Cluster-Level Roles

Define cluster-wide roles for different access levels:

```yaml
# Cluster Admin Role
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: fibonacci-cluster-admin
  labels:
    security.tier: admin
rules:
- apiGroups: ["*"]
  resources: ["*"]
  verbs: ["*"]
- nonResourceURLs: ["*"]
  verbs: ["*"]
---
# Cluster Reader Role
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: fibonacci-cluster-reader
  labels:
    security.tier: reader
rules:
- apiGroups: [""]
  resources: ["pods", "services", "configmaps", "secrets"]
  verbs: ["get", "list", "watch"]
- apiGroups: ["apps"]
  resources: ["deployments", "replicasets"]
  verbs: ["get", "list", "watch"]
- apiGroups: ["monitoring.coreos.com"]
  resources: ["servicemonitors", "prometheusrules"]
  verbs: ["get", "list", "watch"]
```

### 2. Namespace-Specific Roles

Create roles with granular permissions for specific namespaces:

```yaml
# Fibonacci App Management Role
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  namespace: fibonacci-app
  name: fibonacci-app-manager
rules:
- apiGroups: [""]
  resources: ["pods", "services", "configmaps", "secrets", "persistentvolumeclaims"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
- apiGroups: ["apps"]
  resources: ["deployments", "replicasets"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
- apiGroups: ["networking.k8s.io"]
  resources: ["networkpolicies"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
---
# Monitoring Management Role
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  namespace: monitoring
  name: monitoring-manager
rules:
- apiGroups: [""]
  resources: ["pods", "services", "configmaps", "secrets"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
- apiGroups: ["apps"]
  resources: ["deployments", "daemonsets", "statefulsets"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
- apiGroups: ["monitoring.coreos.com"]
  resources: ["servicemonitors", "prometheusrules", "prometheuses", "alertmanagers"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
---
# Application Reader Role
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  namespace: fibonacci-app
  name: fibonacci-app-reader
rules:
- apiGroups: [""]
  resources: ["pods", "services", "configmaps"]
  verbs: ["get", "list", "watch"]
- apiGroups: ["apps"]
  resources: ["deployments", "replicasets"]
  verbs: ["get", "list", "watch"]
- apiGroups: [""]
  resources: ["pods/log"]
  verbs: ["get", "list"]
```

### 3. Role Bindings

Bind roles to users and service accounts:

```yaml
# Admin User Cluster Role Binding
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: fibonacci-admin-binding
  labels:
    security.tier: admin
subjects:
- kind: User
  name: admin
  apiGroup: rbac.authorization.k8s.io
- kind: ServiceAccount
  name: fibonacci-service-account
  namespace: fibonacci-app
roleRef:
  kind: ClusterRole
  name: fibonacci-cluster-admin
  apiGroup: rbac.authorization.k8s.io
---
# Reader User Cluster Role Binding
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: fibonacci-reader-binding
  labels:
    security.tier: reader
subjects:
- kind: User
  name: reader
  apiGroup: rbac.authorization.k8s.io
- kind: Group
  name: readers
  apiGroup: rbac.authorization.k8s.io
roleRef:
  kind: ClusterRole
  name: fibonacci-cluster-reader
  apiGroup: rbac.authorization.k8s.io
---
# Service Account Role Bindings
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: fibonacci-app-manager-binding
  namespace: fibonacci-app
subjects:
- kind: ServiceAccount
  name: fibonacci-service-account
  namespace: fibonacci-app
roleRef:
  kind: Role
  name: fibonacci-app-manager
  apiGroup: rbac.authorization.k8s.io
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: monitoring-manager-binding
  namespace: monitoring
subjects:
- kind: ServiceAccount
  name: prometheus-service-account
  namespace: monitoring
- kind: ServiceAccount
  name: otel-collector-service-account
  namespace: monitoring
roleRef:
  kind: Role
  name: monitoring-manager
  apiGroup: rbac.authorization.k8s.io
```

## Role Definitions

### Admin Role Capabilities
- **Full cluster access**: Can perform any operation on any resource
- **User management**: Can create, modify, and delete user accounts and permissions
- **Resource management**: Can deploy, scale, and delete applications
- **Security management**: Can modify RBAC policies and security configurations
- **Monitoring access**: Full access to all monitoring and logging systems
- **Namespace management**: Can create, modify, and delete namespaces

### Reader Role Capabilities
- **Read-only access**: Can view resources but cannot modify them
- **Log access**: Can view application and system logs
- **Metrics access**: Can view monitoring dashboards and metrics
- **Limited namespace access**: Can only access designated namespaces
- **No security operations**: Cannot view or modify security configurations
- **No user management**: Cannot create or modify user accounts

## Implementation Steps

### Step 1: Update Terraform Configuration

Modify `main.tf` to include RBAC resources:

```hcl
# Add to main.tf

# Create namespaces
resource "kubernetes_namespace" "fibonacci_app" {
  metadata {
    name = "fibonacci-app"
    labels = {
      security-tier = "application"
    }
  }
}

resource "kubernetes_namespace" "monitoring" {
  metadata {
    name = "monitoring"
    labels = {
      security-tier = "monitoring"
    }
  }
}

# Service Accounts
resource "kubernetes_service_account" "fibonacci_sa" {
  metadata {
    name      = "fibonacci-service-account"
    namespace = kubernetes_namespace.fibonacci_app.metadata[0].name
    labels = {
      app           = "fibonacci"
      security-tier = "application"
    }
  }
}

resource "kubernetes_service_account" "prometheus_sa" {
  metadata {
    name      = "prometheus-service-account"
    namespace = kubernetes_namespace.monitoring.metadata[0].name
    labels = {
      app           = "prometheus"
      security-tier = "monitoring"
    }
  }
}

# Cluster Roles
resource "kubernetes_cluster_role" "fibonacci_admin" {
  metadata {
    name = "fibonacci-cluster-admin"
    labels = {
      security-tier = "admin"
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

resource "kubernetes_cluster_role" "fibonacci_reader" {
  metadata {
    name = "fibonacci-cluster-reader"
    labels = {
      security-tier = "reader"
    }
  }

  rule {
    api_groups = [""]
    resources  = ["pods", "services", "configmaps"]
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

# Role Bindings
resource "kubernetes_cluster_role_binding" "fibonacci_admin_binding" {
  metadata {
    name = "fibonacci-admin-binding"
    labels = {
      security-tier = "admin"
    }
  }

  role_ref {
    api_group = "rbac.authorization.k8s.io"
    kind      = "ClusterRole"
    name      = kubernetes_cluster_role.fibonacci_admin.metadata[0].name
  }

  subject {
    kind      = "ServiceAccount"
    name      = kubernetes_service_account.fibonacci_sa.metadata[0].name
    namespace = kubernetes_service_account.fibonacci_sa.metadata[0].namespace
  }
}

resource "kubernetes_cluster_role_binding" "fibonacci_reader_binding" {
  metadata {
    name = "fibonacci-reader-binding"
    labels = {
      security-tier = "reader"
    }
  }

  role_ref {
    api_group = "rbac.authorization.k8s.io"
    kind      = "ClusterRole"
    name      = kubernetes_cluster_role.fibonacci_reader.metadata[0].name
  }

  subject {
    kind      = "User"
    name      = "reader"
    api_group = "rbac.authorization.k8s.io"
  }

  subject {
    kind      = "Group"
    name      = "readers"
    api_group = "rbac.authorization.k8s.io"
  }
}
```

### Step 2: Update Deployment Configurations

Modify the Fibonacci deployment to use the new service account and namespace:

```hcl
# Update the fibonacci deployment in main.tf
resource "kubernetes_deployment" "fibonacci" {
  metadata {
    name      = "fibonacci-deployment"
    namespace = kubernetes_namespace.fibonacci_app.metadata[0].name
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
        service_account_name = kubernetes_service_account.fibonacci_sa.metadata[0].name
        
        # Add security context
        security_context {
          run_as_non_root = true
          run_as_user     = 1000
          run_as_group    = 1000
          fs_group        = 1000
        }

        container {
          name  = "fibonacci"
          image = "${var.docker_username}/fibonacci_rust:${var.docker_image_tag}"

          # Add security context for container
          security_context {
            allow_privilege_escalation = false
            read_only_root_filesystem  = true
            run_as_non_root           = true
            run_as_user               = 1000
            capabilities {
              drop = ["ALL"]
            }
          }

          # ... rest of container configuration
        }
      }
    }
  }
}
```

### Step 3: Implement Network Policies

Add network policies to control traffic between services:

```yaml
# Network policy for Fibonacci app
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: fibonacci-network-policy
  namespace: fibonacci-app
spec:
  podSelector:
    matchLabels:
      app: fibonacci
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          security-tier: monitoring
    ports:
    - protocol: TCP
      port: 8080
  - from: []  # Allow ingress from anywhere for the service
    ports:
    - protocol: TCP
      port: 8080
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          security-tier: monitoring
    ports:
    - protocol: TCP
      port: 8889
  - to: []  # Allow DNS resolution
    ports:
    - protocol: UDP
      port: 53
---
# Network policy for monitoring namespace
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: monitoring-network-policy
  namespace: monitoring
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          security-tier: application
  - from:
    - namespaceSelector:
        matchLabels:
          security-tier: monitoring
  egress:
  - to: []  # Allow all egress for monitoring tools
```

### Step 4: Application-Level Authentication

Implement authentication at the application level by modifying the Rust application:

#### Add Dependencies to Cargo.toml:

```toml
[dependencies]
# ... existing dependencies
jsonwebtoken = "8.3"
serde_json = "1.0"
base64 = "0.21"
uuid = { version = "1.0", features = ["v4", "serde"] }
```

#### Create Authentication Middleware:

```rust
// src/auth.rs
use actix_web::{dev::ServiceRequest, Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

pub async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, Error> {
    let token = credentials.token();
    
    // In production, use a proper secret key
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-secret-key".to_string());
    
    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(token_data) => {
            req.extensions_mut().insert(token_data.claims);
            Ok(req)
        }
        Err(_) => {
            let config = req.app_data::<Config>().map(|data| data.clone()).unwrap_or_else(Default::default);
            Err(AuthenticationError::from(config).into())
        }
    }
}
```

#### Update Main Application:

```rust
// src/main.rs
mod auth;

use actix_web::{web, App, HttpServer, HttpResponse, Result, HttpRequest};
use actix_web_httpauth::middleware::HttpAuthentication;
use auth::{validator, Claims};

async fn fibonacci_protected(req: HttpRequest, path: web::Path<u32>) -> Result<HttpResponse> {
    let claims = req.extensions().get::<Claims>().unwrap();
    
    // Check if user has appropriate role
    match claims.role.as_str() {
        "admin" | "reader" => {
            // Existing fibonacci logic here
            let n = path.into_inner();
            let result = calculate_fibonacci(n);
            Ok(HttpResponse::Ok().json(result))
        }
        _ => Ok(HttpResponse::Forbidden().json("Insufficient permissions")),
    }
}

async fn admin_only_endpoint(req: HttpRequest) -> Result<HttpResponse> {
    let claims = req.extensions().get::<Claims>().unwrap();
    
    if claims.role != "admin" {
        return Ok(HttpResponse::Forbidden().json("Admin access required"));
    }
    
    Ok(HttpResponse::Ok().json("Admin endpoint accessed"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let auth = HttpAuthentication::bearer(validator);
        
        App::new()
            .route("/health", web::get().to(health_check))  // Unprotected health check
            .service(
                web::scope("/api")
                    .wrap(auth)
                    .route("/fibonacci/{n}", web::get().to(fibonacci_protected))
                    .route("/admin", web::get().to(admin_only_endpoint))
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
```

### Step 5: Update Helm Charts

Modify the Helm chart to include RBAC configurations:

```yaml
# fibonacci/templates/rbac.yaml
{{- if .Values.rbac.create -}}
apiVersion: v1
kind: ServiceAccount
metadata:
  name: {{ include "fibonacci.serviceAccountName" . }}
  namespace: {{ .Release.Namespace }}
  labels:
    {{- include "fibonacci.labels" . | nindent 4 }}
  {{- with .Values.serviceAccount.annotations }}
  annotations:
    {{- toYaml . | nindent 4 }}
  {{- end }}
---
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: {{ include "fibonacci.fullname" . }}-role
  namespace: {{ .Release.Namespace }}
rules:
- apiGroups: [""]
  resources: ["pods", "services", "configmaps"]
  verbs: ["get", "list", "watch"]
- apiGroups: ["apps"]
  resources: ["deployments"]
  verbs: ["get", "list", "watch"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: {{ include "fibonacci.fullname" . }}-rolebinding
  namespace: {{ .Release.Namespace }}
subjects:
- kind: ServiceAccount
  name: {{ include "fibonacci.serviceAccountName" . }}
  namespace: {{ .Release.Namespace }}
roleRef:
  kind: Role
  name: {{ include "fibonacci.fullname" . }}-role
  apiGroup: rbac.authorization.k8s.io
{{- end }}
```

Update `fibonacci/values.yaml`:

```yaml
# Add RBAC configuration
rbac:
  create: true

serviceAccount:
  create: true
  annotations: {}
  name: ""

# Add security context
securityContext:
  runAsNonRoot: true
  runAsUser: 1000
  runAsGroup: 1000
  fsGroup: 1000

podSecurityContext:
  allowPrivilegeEscalation: false
  readOnlyRootFilesystem: true
  runAsNonRoot: true
  runAsUser: 1000
  capabilities:
    drop:
    - ALL
```

### Step 6: Environment Configuration

Create environment-specific configuration files:

```yaml
# config/development.yaml
authentication:
  enabled: false
  jwt_secret: "dev-secret-key"

authorization:
  default_role: "reader"
  admin_users:
    - "admin@localhost"

security:
  enforce_https: false
  cors_enabled: true
```

```yaml
# config/production.yaml
authentication:
  enabled: true
  jwt_secret: "${JWT_SECRET}"
  oidc:
    issuer: "${OIDC_ISSUER}"
    client_id: "${OIDC_CLIENT_ID}"

authorization:
  default_role: "reader"
  admin_users:
    - "${ADMIN_USERS}"

security:
  enforce_https: true
  cors_enabled: false
```

## Security Considerations

### 1. Secret Management

- **Never hardcode secrets** in configuration files or code
- Use Kubernetes Secrets for sensitive data
- Consider using a secret management system like HashiCorp Vault
- Rotate secrets regularly

```yaml
# Example secret for JWT signing
apiVersion: v1
kind: Secret
metadata:
  name: fibonacci-jwt-secret
  namespace: fibonacci-app
type: Opaque
stringData:
  jwt-secret: "your-base64-encoded-secret-here"
```

### 2. Pod Security Standards

Implement Pod Security Standards to enhance security:

```yaml
# Pod Security Policy
apiVersion: policy/v1beta1
kind: PodSecurityPolicy
metadata:
  name: fibonacci-psp
spec:
  privileged: false
  allowPrivilegeEscalation: false
  requiredDropCapabilities:
    - ALL
  volumes:
    - 'configMap'
    - 'emptyDir'
    - 'projected'
    - 'secret'
    - 'downwardAPI'
    - 'persistentVolumeClaim'
  runAsUser:
    rule: 'MustRunAsNonRoot'
  seLinux:
    rule: 'RunAsAny'
  fsGroup:
    rule: 'RunAsAny'
```

### 3. Network Security

- Implement network policies to restrict communication
- Use TLS for all inter-service communication
- Consider service mesh (like Istio) for advanced traffic management

### 4. Audit Logging

Enable audit logging to track all API calls:

```yaml
# Audit policy
apiVersion: audit.k8s.io/v1
kind: Policy
rules:
- level: Metadata
  namespaces: ["fibonacci-app", "monitoring"]
  verbs: ["get", "list", "create", "update", "patch", "delete"]
```

## Testing and Verification

### 1. Test RBAC Permissions

Create test scripts to verify role permissions:

```bash
#!/bin/bash
# test-rbac.sh

echo "Testing admin permissions..."
kubectl auth can-i create deployments --as=admin
kubectl auth can-i delete pods --as=admin

echo "Testing reader permissions..."
kubectl auth can-i get pods --as=reader
kubectl auth can-i create deployments --as=reader  # Should be "no"
kubectl auth can-i delete pods --as=reader  # Should be "no"

echo "Testing service account permissions..."
kubectl auth can-i get pods --as=system:serviceaccount:fibonacci-app:fibonacci-service-account
```

### 2. Test Authentication

```bash
#!/bin/bash
# test-auth.sh

echo "Testing unauthenticated access..."
curl -i http://localhost:8080/api/fibonacci/10  # Should return 401

echo "Testing authenticated access..."
# Get token first (implementation depends on your auth method)
TOKEN=$(get_jwt_token_for_user "reader")
curl -i -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/fibonacci/10

echo "Testing admin endpoint..."
ADMIN_TOKEN=$(get_jwt_token_for_user "admin")
curl -i -H "Authorization: Bearer $ADMIN_TOKEN" http://localhost:8080/api/admin
```

### 3. Security Validation

```bash
#!/bin/bash
# security-validation.sh

echo "Checking for privileged containers..."
kubectl get pods -o jsonpath='{.items[*].spec.containers[*].securityContext.privileged}' | grep -v true

echo "Checking for containers running as root..."
kubectl get pods -o jsonpath='{.items[*].spec.containers[*].securityContext.runAsUser}' | grep -v 0

echo "Validating network policies..."
kubectl get networkpolicies -A

echo "Checking RBAC bindings..."
kubectl get rolebindings,clusterrolebindings -A
```

## Troubleshooting

### Common Issues and Solutions

#### 1. Permission Denied Errors

```bash
# Check current user permissions
kubectl auth can-i <verb> <resource> --as=<user>

# Check role bindings
kubectl get rolebindings,clusterrolebindings -A -o wide

# Verify service account exists
kubectl get serviceaccounts -n <namespace>
```

#### 2. Authentication Failures

```bash
# Check JWT token validity
echo $TOKEN | base64 -d | jq .

# Verify certificate configuration
openssl x509 -in <cert-file> -text -noout

# Check OIDC configuration
kubectl get configmap oidc-config -n kube-system -o yaml
```

#### 3. Network Policy Issues

```bash
# Test connectivity between pods
kubectl exec -it <pod-name> -- nc -zv <target-service> <port>

# Check network policy configuration
kubectl describe networkpolicy <policy-name> -n <namespace>

# Verify DNS resolution
kubectl exec -it <pod-name> -- nslookup <service-name>
```

#### 4. Service Account Token Issues

```bash
# Check service account token
kubectl get secret $(kubectl get serviceaccount <sa-name> -o jsonpath='{.secrets[0].name}') -o yaml

# Verify token mount in pod
kubectl exec -it <pod-name> -- ls -la /var/run/secrets/kubernetes.io/serviceaccount/
```

### Monitoring and Alerting

Set up monitoring for authentication and authorization events:

```yaml
# Prometheus alerting rules
groups:
- name: kubernetes.auth
  rules:
  - alert: AuthenticationFailures
    expr: increase(apiserver_audit_total{verb!="get",objectRef_resource!="events"}[5m]) > 10
    for: 2m
    labels:
      severity: warning
    annotations:
      summary: "High number of authentication failures"
      
  - alert: UnauthorizedAccess
    expr: increase(apiserver_audit_total{verb="create",code!~"2.."}[5m]) > 5
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "Unauthorized access attempts detected"
```

## Deployment Checklist

Before deploying authentication and authorization:

- [ ] Backup existing cluster configuration
- [ ] Test RBAC policies in development environment
- [ ] Verify service account permissions
- [ ] Test authentication mechanisms
- [ ] Validate network policies
- [ ] Configure monitoring and alerting
- [ ] Update documentation
- [ ] Train team on new security procedures
- [ ] Plan rollback strategy
- [ ] Schedule deployment during maintenance window

## Additional Resources

- [Kubernetes RBAC Documentation](https://kubernetes.io/docs/reference/access-authn-authz/rbac/)
- [Pod Security Standards](https://kubernetes.io/docs/concepts/security/pod-security-standards/)
- [Network Policies](https://kubernetes.io/docs/concepts/services-networking/network-policies/)
- [Audit Logging](https://kubernetes.io/docs/tasks/debug-application-cluster/audit/)
- [Security Best Practices](https://kubernetes.io/docs/concepts/security/)

This comprehensive guide provides all the necessary instructions for implementing robust authentication and authorization in the Fibonacci Kubernetes cluster. Follow each step carefully and test thoroughly in a development environment before applying to production.