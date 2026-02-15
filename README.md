# Tech Stack used in this Project
![Rust_programming_language_black_logo.svg.png](img/Rust_programming_language_black_logo.svg.png)
![Terraform_Logo.png](img/Terraform_Logo.png)
![Kubernetes_logo_without_workmark.svg.png](img/Kubernetes_logo_without_workmark.svg.png)
![Prometheus_software_logo.svg.png](img/Prometheus_software_logo.svg.png)
![Grafana_logo.svg.png](img/Grafana_logo.svg.png)
![Docker_logo.png](img/Docker_logo.png)

# Rust App that calculates the Fibonacci sequence and deploys it to Kubernetes 
[![Rust](https://github.com/paraskuk/Rust-Fibonacci-Kubernetes-Prometheus-Grafana/actions/workflows/rust.yml/badge.svg)](https://github.com/paraskuk/Rust-Fibonacci-Kubernetes-Prometheus-Grafana/actions/workflows/rust.yml)

> [!TIP]
> * This project provides various implementations of the Fibonacci sequence in Rust, including recursive, memoized, iterative, and dynamic programming approaches. 
> * It also includes a Kubernetes deployment with Helm charts deploying to minikube.
> * Cluster has also Prometheus and Grafana installed to monitor the application.


> [!TIP] 
> The Fibonacci sequence is a series of numbers in which each number is the sum of the two preceding ones, usually starting with 0 and 1.

> [!TIP]
> The sequence starts: 0, 1, 1, 2, 3, 5, 8, 13, 21, 34, ...

> [!TIP]
> The Fibonacci sequence is defined by the recurrence relation: F(n) = F(n-1) + F(n-2) with base cases F(0) = 0 and F(1) = 1.

> [!TIP]
> The Fibonacci sequence grows exponentially, so the recursive implementation is not efficient for large values of n.

> [!TIP]
> The iterative and dynamic programming implementations are more efficient for large values of n.

> [!TIP]
> The memoized recursive implementation is more efficient than the simple recursive implementation because it avoids redundant calculations.

> [!TIP]
> The pattern matching implementation is similar to the simple recursive implementation but uses pattern matching instead of if-else statements.

> [!TIP]
> The dynamic programming implementation uses an array to store the Fibonacci numbers and avoids redundant calculations.

The following implementations are provided:

- `fibonacci`: A simple recursive implementation.
- `fibonacci_match`: A recursive implementation using pattern matching.
- `fibonacci_dp`: An implementation using dynamic programming.
- `fibonacci_memo`: A memoized recursive implementation.
- `fibonacci_iterative`: An iterative implementation.


## Prerequisites
> [!IMPORTANT]
> Before you begin, ensure you have the following tools installed:
> 1. **Rust & Cargo** (for local builds/verification, optional if you only build in Docker).
> 2. **Docker** (to build and push container images).
> 3. **Helm** (to manage Kubernetes deployments).
> 4. **Minikube or a Kubernetes Cluster** (for testing).
> 5. **Docker Hub Account** (to push container images).

---

## How to Run the Application

### Option 1: Run Locally (Development)
> [!TIP]
> This is the fastest way to test changes during development. Make sure you have Rust and Cargo installed.

**Step 1: Build the project**
```cmd
cargo build
```

**Step 2: Run the application**
```cmd
cargo run
```

**Step 3: Access the application**
- Open your browser and navigate to `http://localhost:8080`
- The application will start on port 8080

**Step 4: Test the Fibonacci API**
You can use curl or Postman to test the endpoints:
```cmd
curl -X POST http://localhost:8080/fib -H "Content-Type: application/json" -d "{\"n\": 10}"
```

**Step 5: Check application health**
```cmd
curl http://localhost:8080/health
curl http://localhost:8080/ready
```

**Step 6: View metrics (Prometheus format)**
```cmd
curl http://localhost:8080/metrics
```

> [!NOTE]
> The application creates a log file at `/var/log/fibonacci.log` when running with log4rs configured.

### Option 2: Run in Docker (Containerized)
> [!TIP]
> Use this approach for consistent environment across machines and to simulate production-like deployment.

**Step 1: Build the Docker image**
```cmd
docker build -t fibonacci_rust:latest .
```

**Step 2: Run the Docker container**
```cmd
docker run -p 8080:8080 --name fibonacci fibonacci_rust:latest
```

**Step 3: Access the application**
- Open your browser and navigate to `http://localhost:8080`

**Step 4: View container logs**
```cmd
docker logs fibonacci
```

**Step 5: Stop and remove the container**
```cmd
docker stop fibonacci
docker rm fibonacci
```

#### Push to Docker Hub
```cmd
docker login --username your_docker_username
docker tag fibonacci_rust:latest your_docker_username/fibonacci_rust:v1.0
docker push your_docker_username/fibonacci_rust:v1.0
```

#### Push to Azure Container Registry (ACR)
```cmd
az login
az acr login --name your_registry_name
docker tag fibonacci_rust:latest your_registry_name.azurecr.io/fibonacci_rust:v1.0
docker push your_registry_name.azurecr.io/fibonacci_rust:v1.0
```

#### Push to Google Container Registry (GCR)
```cmd
gcloud auth login
gcloud auth configure-docker
docker tag fibonacci_rust:latest gcr.io/your_project_id/fibonacci_rust:v1.0
docker push gcr.io/your_project_id/fibonacci_rust:v1.0
```

#### Push to AWS Elastic Container Registry (ECR)
```cmd
aws ecr get-login-password --region your_aws_region | docker login --username AWS --password-stdin your_account_id.dkr.ecr.your_aws_region.amazonaws.com
docker tag fibonacci_rust:latest your_account_id.dkr.ecr.your_aws_region.amazonaws.com/fibonacci_rust:v1.0
docker push your_account_id.dkr.ecr.your_aws_region.amazonaws.com/fibonacci_rust:v1.0
```

#### Push to Private Registry
```cmd
docker login your_private_registry_url
docker tag fibonacci_rust:latest your_private_registry_url/fibonacci_rust:v1.0
docker push your_private_registry_url/fibonacci_rust:v1.0
```

### Option 3: Run in Kubernetes with Helm (Production)
> [!TIP]
> Use this approach for production deployments with Prometheus monitoring and Grafana visualization.

#### Docker Image Available on Docker Hub
> [!IMPORTANT]
> The image has been successfully pushed to Docker Hub:
> - **Image**: `paraskevas68/fibonacci_rust:v104`
> - **Version**: v104 ✅ (Latest - Clean & Fixed)
> - **Previous Versions**: 
>   - v103 - Ultra Fun Edition (rainbow theme)
>   - v102 - Modern UI with Bootstrap 5
>   - v101 - Fixed web UI response handling
>   - v100 - Initial release
> - **Access**: Available at [Docker Hub Registry](https://hub.docker.com/r/paraskevas68/fibonacci_rust)
> - **Pull Command**: `docker pull paraskevas68/fibonacci_rust:v104`
> 
> **✅ Changes in v104 - CLEAN & WORKING**: 
> - 🎯 **FIXED**: Result now displays correctly every time!
> - 📐 **Vertical Layout** - Clean, organized vertical sections
> - 🧹 **No Duplicated Content** - Fixed HTML corruption
> - 🚫 **No Rainbow Symbols** - Professional, clean design
> - 🎨 **Modern Gradient UI** - Purple/blue gradient background
> - 💎 **Bootstrap 5** - Professional styling
> - 📱 **Fully Responsive** - Works on all devices
> - ⚡ **Fast & Lightweight** - Optimized performance
> - 🎯 **Loading State** - Visual feedback during calculation
> - ✨ **Smooth Animations** - Professional transitions

See the detailed instructions in sections [3. Deploying the Application to Kubernetes](#3-deploying-the-application-to-kubernetes) below.

---

## 1. Building the Docker Image
> [!TIP]
> You can build the Docker image using the provided Dockerfile, tag it, and push to Docker Hub.
> The latest version v104 features a clean, professional UI with working result display!
```cmd
docker login --username paraskevas68
docker build -t paraskevas68/fibonacci_rust:v104 .
docker push paraskevas68/fibonacci_rust:v104
```

> [!NOTE]
> **Version History**:
> - **v104** ✅ - Latest (Clean UI - Fixed result display, vertical layout, no duplicates)
> - **v103** - Ultra Fun Edition (rainbow animations, confetti)
> - **v102** - Modern UI with Bootstrap 5
> - **v101** - Fixed web UI response handling
> - **v100** - Initial release

> [!TIP]
> **✨ UI Features in v104 - CLEAN & PROFESSIONAL**:
> - ✅ **Working Result Display** - Shows Fibonacci results correctly
> - 📐 **Vertical Layout** - Clean, organized sections
> - 🎨 **Modern Gradient** - Purple/blue gradient background
> - 💎 **Bootstrap 5** - Professional component library
> - 📱 **Responsive Design** - Mobile, tablet, desktop optimized
> - ⚡ **Loading States** - Visual feedback during calculations
> - 🎯 **Form Validation** - Real-time input validation (1-50)
> - ✨ **Smooth Animations** - Professional transitions
> - 🚫 **No Clutter** - Clean, focused design without rainbow symbols

## 2. Deploying the Application to Kubernetes via Terraform
> [!TIP]
> This repository includes a simple Terraform module (in the terraform/ directory) that deploys the fibonacci container into Kubernetes. The key files are:
> - main.tf – Terraform configuration.
> - Supports deploying the Docker Hub v104 image with clean UI

### 2.1 Deploy the Application
> [!TIP]
> Navigate to the terraform/ directory and run the following commands to deploy the application with the v104 image:
```cmd
terraform init
```

```cmd
terraform validate
```

```cmd
terraform apply -var="docker_username=paraskevas68" -var="docker_image_tag=v104"
```

- Once all resources are deployed you can start the minikube service to access the application.
```cmd
minikube service fibonacci-service
```

- You can go to this section to see about monitoring [3.5 Monitoring the Application with Prometheus and Grafana](#35-monitoring-the-application-with-prometheus-and-grafana)


## 3. Deploying the Application to Kubernetes
### 3.1 Deploy via Helm Charts - Helm Chart Overview
> [!TIP]
> This repository includes a simple Helm chart (in the fibonacci/ directory, for example) that deploys the fibonacci container into Kubernetes. The key files are:
> - Chart.yaml – Chart metadata. 
> - values.yaml – Default values (e.g., image, replicas, etc.).
> - templates/ – Contains Kubernetes manifests (Deployment, Service, etc.).

### 3.2 Creating Secret
> [!TIP]
> Create a secret to pull the image from Docker Hub:
```sh
kubectl create secret docker-registry regcred \
--docker-server=docker.io \
--docker-username=myusername \
--docker-password=MY_PERSONAL_ACCESS_TOKEN \
--docker-email=myemail@example.com
```

Then reference it in your values.yaml or deployment.yaml:
> [!TIP]
> Add the following to your deployment.yaml file:
```yaml
imagePullSecrets:
- name: regcred
```

### 3.3 Install Helm Chart
> [!TIP]
> Navigate to the fibonacci/ directory and install the Helm chart for the first time using the v104 image:
```cmd
helm install fibonacci . --set image.repository=paraskevas68/fibonacci_rust --set image.tag=v104
```
> [!WARNING]
> Do not execute helm install again as it will overwrite the existing deployment.

> [!TIP]
> If you already have the chart installed and want to upgrade it with the v104 image:

```cmd
helm upgrade --install fibonacci . --set image.repository=paraskevas68/fibonacci_rust --set image.tag=v104
```
> [!TIP]
> Check the deployment status:

```cmd
helm status fibonacci
kubectl get pods
kubectl get services
kubectl get deployments
```
### 3.4 Launching the Application
> [!TIP]
> Launch a service to expose the application. Your default web browser will open the application.:

```cmd
minikube service fibonacci-service
```
> [!TIP]
> You can access the logs of the pod in /var/log/fibonacci.log
> go inside the pod and check the logs
```cmd
kubectl exec -it <your-pod-name> -- /bin/sh
cat fibonacci.log
```
### 3.5 Monitoring the Application with Prometheus and Grafana
> [!TIP]
> You can monitor the application using Prometheus and Grafana.

> [!TIP]
> Access the Prometheus dashboard at http://localhost:9090 and the Grafana dashboard at http://localhost:3000.

> [!IMPORTANT]
> You need to port forward the Prometheus and Grafana services to access them locally:

```cmd
kubectl port-forward svc/prometheus 9090:9090
```

```cmd
kubectl port-forward svc/grafana 3000:3000
```

> [!TIP]
> Go to http://localhost:9090 to access the Prometheus dashboard.
> You will see a similar picture as below if you look for the requests_total metric:![prometheus.png](img/prometheus.png)

> [!TIP]
> Go to http://localhost:3000 to access the Grafana dashboard.
> Login with the default username and password (admin/admin).
> Get your IP with the following command:
> 
```cmd
kubectl get svc prometheus -n default -o jsonpath='{.spec.clusterIP}'
```
> [!TIP]
> Go to the Grafana dashboard and add a new data source with the IP address you just got from the previous command.
```cmd
http://<your-ip>:9090
```
> [!TIP]
> You will see a similar picture as below if you look for the requests_total metric:![Grafana.png](img/Grafana.png)

### 3. Cleaning Up
> [!WARNING]
> This will uninstall helm release.

```cmd
helm uninstall fibonacci
```

> [!CAUTION]
> This will delete the deployment, secret, service and the minikube cluster.

```cmd
kubectl delete secret regcred
minikube delete
```
