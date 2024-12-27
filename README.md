
# Rust App that calculates the Fibonacci sequence and deploys it to Kubernetes 

This project provides various implementations of the Fibonacci sequence in Rust, including recursive, memoized, iterative, and dynamic programming approaches. 
It also includes a Kubernetes deployment using Helm charts using minikube.

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

> [!TIP]
> The following implementations are provided:

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
docker tag fibonacci_rust:latest ${DOCKER_USERNAME}/fibonacci_rust:v24
docker push ${DOCKER_USERNAME}/fibonacci_rust:v24
```

## 2. Deploying the Application to Kubernetes
### 2.0 Deploy using Manifests
> [!TIP]
> You can deploy the application to Kubernetes using the provided deployment.yaml file:
```sh
kubectl apply -f deployment.yaml
```

### 2.1 Or Deploy via Helm Charts - Helm Chart Overview
> [!TIP]
> This repository includes a simple Helm chart (in the fibonacci/ directory, for example) that deploys the fibonacci container into Kubernetes. The key files are:

* Chart.yaml – Chart metadata.
* values.yaml – Default values (e.g., image, replicas, etc.).
* templates/ – Contains Kubernetes manifests (Deployment, Service, etc.).

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
helm install fibonacci .
```
> [!WARNING]
> Do not execute helm install again as it will overwrite the existing deployment.

> [!TIP]
> If you already have the chart installed and want to upgrade it:

```sh
helm upgrade --install fibonacci .
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
> go to inside the pod and check the logs
```sh
kubectl exec -it <your-pod-name>  -- /bin/sh
 cat fibonacci.log
```
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
