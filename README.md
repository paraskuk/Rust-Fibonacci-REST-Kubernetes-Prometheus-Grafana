[![Rust](https://github.com/paraskuk/Fibonacci-Rust-Kubernetes-Deployment-Command-Line-POC-App/actions/workflows/rust.yml/badge.svg)](https://github.com/paraskuk/Fibonacci-Rust-Kubernetes-Deployment-Command-Line-POC-App/actions/workflows/rust.yml)
# Fibonacci-Rust-Kubernetes-Deployment-Command-Line-POC-App

This project provides various implementations of the Fibonacci sequence in Rust, including recursive, memoized, iterative, and dynamic programming approaches. 
It also includes a Kubernetes deployment using Helm charts.

## Implementations

- `fibonacci`: A simple recursive implementation.
- `fibonacci_match`: A recursive implementation using pattern matching.
- `fibonacci_dp`: An implementation using dynamic programming.
- `fibonacci_memo`: A memoized recursive implementation.
- `fibonacci_iterative`: An iterative implementation.


## Prerequisites

1. **Rust & Cargo** (for local builds/verification, optional if you only build in Docker).
2. **Docker** (to build and push container images).
3. **Helm** (to manage Kubernetes deployments).
4. **Minikube or a Kubernetes Cluster** (for testing).
5. **Docker Hub Account** (to push container images).

---

## 1. Building the Rust Application Locally (Optional)

You can verify that the application compiles and runs locally:

```sh
cargo run --package fibonacci --bin fibonacci -- 10

# Output:
Time taken by fibonacci_match: ...
Time taken by fibonacci_memo: ...
Time taken by fibonacci_iterative: ...
```

## 2. Building the Docker Image
```sh
docker build -t fibonacci_rust:latest .
docker tag fibonacci_rust:latest myusername/fibonacci_rust:latest
docker login --username myusername
docker push myusername/fibonacci_rust:latest
docker tag fibonacci_rust:latest myusername/fibonacci_rust:v2
docker push myusername/fibonacci_rust:v2
```

## 3. Deploying the Application to Kubernetes
### 3.0 Deploy using Manifests
```sh
kubectl apply -f deployment.yaml
```

### 3.1 Or Deploy via Helm Charts - Helm Chart Overview
* This repository includes a simple Helm chart (in the fibonacci/ directory, for example) that deploys the fibonacci container into Kubernetes. The key files are:

* Chart.yaml – Chart metadata.
* values.yaml – Default values (e.g., image, replicas, etc.).
* templates/ – Contains Kubernetes manifests (Deployment, Service, etc.).

### 3.2 Creating Secret
```sh
kubectl create secret docker-registry regcred \
--docker-server=docker.io \
--docker-username=myusername \
--docker-password=MY_PERSONAL_ACCESS_TOKEN \
--docker-email=myemail@example.com
```

Then reference it in your values.yaml or deployment.yaml:
```yaml
imagePullSecrets:
- name: regcred
```

### 3.3 Install Helm Chart
Navigate to the fibonacci/ directory and install the Helm chart:
For the first time:
```sh
helm install fibonacci .
```
If you already have the chart installed and want to upgrade it:
```sh
helm upgrade --install fibonacci .
```
Check the deployment status:

```sh
helm status fibonacci
kubectl get pods
```
### 3.4 Accessing the Application
This program just prints to the STDOUT
```sh
kubectl logs deployment/fibonacci-deployment
```

### 4 .Cleaning Up
```sh
helm uninstall fibonacci
```

```sh
kubectl delete secret regcred
minkube delete
```
