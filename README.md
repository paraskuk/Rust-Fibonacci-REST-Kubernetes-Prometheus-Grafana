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



## 1. Building the Docker Image
> [!TIP]
> You can build the Docker image using the provided Dockerfile
> Tag the image after its building
> Using your username login to Docker Hub and push the image to Docker Hub.
```sh
docker login --username ${DOCKER_USERNAME}
docker build -t fibonacci_rust:latest .
docker tag fibonacci_rust:latest ${DOCKER_USERNAME}/fibonacci_rust:${DOCKER_TAG}
docker push ${DOCKER_USERNAME}/fibonacci_rust:${DOCKER_TAG}
```

## 1. Deploying the Application to Kubernetes via Terraform
> [!TIP]
> This repository includes a simple Terraform module (in the terraform/ directory) that deploys the fibonacci container into Kubernetes. The key files are:
> - main.tf – Terraform configuration.

### 1.1 Deploy the Application
> [!TIP]
> Navigate to the terraform/ directory and run the following commands to deploy the application:
```sh
terraform init
```

```sh
terraform validate
```

```sh
terraform apply -var="docker_username=your_docker_username" -var="docker_image_tag=your_docker_image_tag"
```

- Once all resources are deployed you can start the minikube service to access the application.
```sh
minikube service fibonacci-service
```

- You can go to this section to see about monitoring [2.5 Monitoring the Application with Prometheus and Grafana](#25-monitoring-the-application-with-prometheus-and-grafana)
 


## 2. Deploying the Application to Kubernetes
### 2.1 Deploy via Helm Charts - Helm Chart Overview
> [!TIP]
> This repository includes a simple Helm chart (in the fibonacci/ directory, for example) that deploys the fibonacci container into Kubernetes. The key files are:
> - Chart.yaml – Chart metadata. 
> - values.yaml – Default values (e.g., image, replicas, etc.).
> - templates/ – Contains Kubernetes manifests (Deployment, Service, etc.).

### 2.2 Creating Secret
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

### 2.3 Install Helm Chart
> [!TIP]
> Navigate to the fibonacci/ directory and install the Helm chart for the first time:
```sh
helm install fibonacci . --set image.repository=${DOCKER_USERNAME}/fibonacci_rust --set image.tag=${DOCKER_TAG}
```
> [!WARNING]
> Do not execute helm install again as it will overwrite the existing deployment.

> [!TIP]
> If you already have the chart installed and want to upgrade it:

```sh
helm upgrade --install fibonacci . --set image.repository=${DOCKER_USERNAME}/fibonacci_rust --set image.tag=${DOCKER_TAG}
```
> [!TIP]
> Check the deployment status:

```sh
helm status fibonacci
kubectl get pods
kubectl get services
kubectl get deployments
```
### 2.4 Launching the Application
> [!TIP]
> Launch a service to expose the application. Your default web browser will open the application.:

```sh
minikube service fibonacci-service
```
> [!TIP]
> You can access the logs of the pod in /var/log/fibonacci.log
> go inside the pod and check the logs
```sh
kubectl exec -it <your-pod-name>  -- /bin/sh
cat fibonacci.log
```
### 2.5 Monitoring the Application with Prometheus and Grafana
> [!TIP]
> You can monitor the application using Prometheus and Grafana.

> [!TIP]
> Access the Prometheus dashboard at http://localhost:9090 and the Grafana dashboard at http://localhost:3000.

> [!IMPORTANT]
> You need to port forward the Prometheus and Grafana services to access them locally:

```sh
kubectl port-forward svc/prometheus 9090:9090
```

```sh
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
```sh
kubectl get svc prometheus -n default -o jsonpath='{.spec.clusterIP}'
```
> [!TIP]
> Go to the Grafana dashboard and add a new data source with the IP address you just got from the previous command.
```sh
http://<your-ip>:9090
```
> [!TIP]
> You will see a similar picture as below if you look for the requests_total metric:![Grafana.png](img/Grafana.png)


### 3 .Cleaning Up
> [!WARNING]
> This will uninstall helm release.

```sh
helm uninstall fibonacci
```

> [!CAUTION]
> This will delete the deployment, secret, service and the minikube cluster.

```sh
kubectl delete secret regcred
minkube delete
```
