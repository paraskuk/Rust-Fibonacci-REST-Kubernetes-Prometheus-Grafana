# Quick Start Authentication Setup

This document provides a streamlined implementation guide for quickly setting up basic authentication and authorization in the Fibonacci Kubernetes cluster.

## Prerequisites

- Kubernetes cluster with RBAC enabled
- `kubectl` with cluster-admin access
- Current Fibonacci application deployed

## 1. Create Namespaces and Service Accounts

```bash
# Create namespaces
kubectl create namespace fibonacci-app
kubectl create namespace monitoring

# Create service accounts
kubectl create serviceaccount fibonacci-service-account -n fibonacci-app
kubectl create serviceaccount prometheus-service-account -n monitoring
```

## 2. Define Basic Roles

Create `rbac-basic.yaml`:

```yaml
# Admin ClusterRole
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: fibonacci-admin
rules:
- apiGroups: ["*"]
  resources: ["*"]
  verbs: ["*"]
---
# Reader ClusterRole  
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: fibonacci-reader
rules:
- apiGroups: [""]
  resources: ["pods", "services", "configmaps"]
  verbs: ["get", "list", "watch"]
- apiGroups: ["apps"] 
  resources: ["deployments", "replicasets"]
  verbs: ["get", "list", "watch"]
- apiGroups: [""]
  resources: ["pods/log"]
  verbs: ["get"]
---
# Bind admin role
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: fibonacci-admin-binding
subjects:
- kind: User
  name: admin
  apiGroup: rbac.authorization.k8s.io
roleRef:
  kind: ClusterRole
  name: fibonacci-admin
  apiGroup: rbac.authorization.k8s.io
---
# Bind reader role
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: fibonacci-reader-binding
subjects:
- kind: User
  name: reader
  apiGroup: rbac.authorization.k8s.io
roleRef:
  kind: ClusterRole
  name: fibonacci-reader
  apiGroup: rbac.authorization.k8s.io
```

Apply the configuration:
```bash
kubectl apply -f rbac-basic.yaml
```

## 3. Update Existing Deployments

Move existing deployments to new namespaces:

```bash
# Export current fibonacci deployment
kubectl get deployment fibonacci-deployment -o yaml > fibonacci-deployment.yaml

# Edit the file to:
# 1. Change namespace to "fibonacci-app"
# 2. Add serviceAccountName: fibonacci-service-account
# 3. Add security context

# Delete old deployment and create new one
kubectl delete deployment fibonacci-deployment
kubectl apply -f fibonacci-deployment.yaml -n fibonacci-app

# Do the same for services
kubectl get service fibonacci-service -o yaml > fibonacci-service.yaml
# Edit namespace and apply
kubectl delete service fibonacci-service
kubectl apply -f fibonacci-service.yaml -n fibonacci-app
```

## 4. Test Permissions

```bash
# Test admin permissions (should work)
kubectl auth can-i create deployments --as=admin

# Test reader permissions (should fail)
kubectl auth can-i create deployments --as=reader

# Test reader read access (should work)
kubectl auth can-i get pods --as=reader
```

## 5. Create User Certificates (Optional)

```bash
# Generate admin certificate
openssl genrsa -out admin.key 2048
openssl req -new -key admin.key -out admin.csr -subj "/CN=admin/O=system:masters"

# Generate reader certificate  
openssl genrsa -out reader.key 2048
openssl req -new -key reader.key -out reader.csr -subj "/CN=reader"

# Sign certificates (requires cluster CA)
openssl x509 -req -in admin.csr -CA /etc/kubernetes/pki/ca.crt -CAkey /etc/kubernetes/pki/ca.key -CAcreateserial -out admin.crt -days 365
openssl x509 -req -in reader.csr -CA /etc/kubernetes/pki/ca.crt -CAkey /etc/kubernetes/pki/ca.key -CAcreateserial -out reader.crt -days 365
```

## 6. Update kubeconfig

```bash
# Add admin user
kubectl config set-credentials admin --client-certificate=admin.crt --client-key=admin.key
kubectl config set-context admin-context --cluster=kubernetes --user=admin

# Add reader user
kubectl config set-credentials reader --client-certificate=reader.crt --client-key=reader.key
kubectl config set-context reader-context --cluster=kubernetes --user=reader

# Test contexts
kubectl config use-context admin-context
kubectl get pods -A  # Should work

kubectl config use-context reader-context
kubectl get pods -A  # Should work (read-only)
kubectl create deployment test --image=nginx  # Should fail
```

## 7. Basic Network Policy

Create `network-policy-basic.yaml`:

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: fibonacci-netpol
  namespace: fibonacci-app
spec:
  podSelector:
    matchLabels:
      app: fibonacci
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from: []  # Allow all ingress
    ports:
    - protocol: TCP
      port: 8080
  egress:
  - to: []  # Allow all egress
```

Apply:
```bash
kubectl apply -f network-policy-basic.yaml
```

## 8. Verification

```bash
# Check RBAC setup
kubectl get clusterroles | grep fibonacci
kubectl get clusterrolebindings | grep fibonacci

# Check deployments in new namespaces
kubectl get deployments -n fibonacci-app
kubectl get deployments -n monitoring

# Test application access
kubectl port-forward svc/fibonacci-service 8080:8080 -n fibonacci-app
curl http://localhost:8080/fibonacci?n=10
```

This quick setup provides basic authentication and authorization. For production use, refer to the comprehensive guide in `KUBERNETES_AUTH_GUIDE.md`.