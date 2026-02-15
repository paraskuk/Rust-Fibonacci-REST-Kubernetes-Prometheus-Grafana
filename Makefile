 
.PHONY: help check-grafana check-prometheus check-fibonacci check-all grafana prometheus fibonacci all traffic dashboard setup-datasource import-dashboard clean quickstart restart restart-all

# Default target
help:
	@echo "=========================================="
	@echo "Fibonacci Service - Grafana Setup"
	@echo "=========================================="
	@echo ""
	@echo "Available targets:"
	@echo "  make check-all         - Check if all services are running"
	@echo "  make grafana          - Port-forward Grafana (http://localhost:3000)"
	@echo "  make prometheus       - Port-forward Prometheus (http://localhost:9090)"
	@echo "  make fibonacci        - Port-forward Fibonacci Service (http://localhost:8080)"
	@echo "  make all              - Port-forward all services in background"
	@echo "  make traffic          - Generate 100 test requests"
	@echo "  make dashboard        - Show dashboard information"
	@echo "  make setup-datasource - Instructions for Prometheus data source setup"
	@echo "  make import-dashboard - Instructions for importing dashboard"
	@echo "  make restart          - Restart Grafana and Prometheus"
	@echo "  make restart-all      - Restart all services (Grafana, Prometheus, Fibonacci)"
	@echo "  make clean            - Stop all port-forwards"
	@echo "  make quickstart       - Show quick start guide"
	@echo ""

# Check if services are running
check-grafana:
	@echo "Checking if Grafana pod is running..."
	@kubectl get pods -l app=grafana || (echo "ERROR: Grafana pod not found" && exit 1)

check-prometheus:
	@echo "Checking if Prometheus pod is running..."
	@kubectl get pods -l app=prometheus || (echo "ERROR: Prometheus pod not found" && exit 1)

check-fibonacci:
	@echo "Checking if Fibonacci service is running..."
	@kubectl get svc fibonacci-service || (echo "ERROR: Fibonacci service not found" && exit 1)

check-all: check-grafana check-prometheus check-fibonacci
	@echo ""
	@echo "=========================================="
	@echo "All services are running!"
	@echo "=========================================="

# Port-forward Grafana
grafana: check-grafana
	@echo ""
	@echo "Starting port-forward for Grafana..."
	@echo "Access Grafana at: http://localhost:3000"
	@echo "Default credentials: admin / admin"
	@echo ""
	@echo "Press Ctrl+C to stop port-forwarding"
	kubectl port-forward deployment/grafana 3000:3000

# Port-forward Prometheus
prometheus: check-prometheus
	@echo ""
	@echo "Starting port-forward for Prometheus..."
	@echo "Access Prometheus at: http://localhost:9090"
	@echo ""
	@echo "Press Ctrl+C to stop port-forwarding"
	kubectl port-forward svc/prometheus 9090:9090

# Port-forward Fibonacci Service
fibonacci: check-fibonacci
	@echo ""
	@echo "Starting port-forward for Fibonacci Service..."
	@echo "Access Fibonacci at: http://localhost:8080"
	@echo "Metrics endpoint: http://localhost:8080/metrics"
	@echo ""
	@echo "Press Ctrl+C to stop port-forwarding"
	kubectl port-forward svc/fibonacci-service 8080:8080

# Port-forward all services in background (Windows version)
all: check-all
	@echo ""
	@echo "Starting port-forward for ALL services..."
	@echo "This will open 3 command windows:"
	@echo "  - Grafana: http://localhost:3000"
	@echo "  - Prometheus: http://localhost:9090"
	@echo "  - Fibonacci: http://localhost:8080"
	@echo ""
	start "Grafana Port-Forward" cmd /k kubectl port-forward deployment/grafana 3000:3000
	timeout /t 2 /nobreak >nul
	start "Prometheus Port-Forward" cmd /k kubectl port-forward svc/prometheus 9090:9090
	timeout /t 2 /nobreak >nul
	start "Fibonacci Port-Forward" cmd /k kubectl port-forward svc/fibonacci-service 8080:8080
	@echo ""
	@echo "All port-forwards started in separate windows!"
	@echo "You can now access:"
	@echo "  - Grafana: http://localhost:3000 (admin/admin)"
	@echo "  - Prometheus: http://localhost:9090"
	@echo "  - Fibonacci: http://localhost:8080"
	@echo ""
	@echo "To stop all port-forwards, run: make clean"

# Generate test traffic
traffic:
	@echo ""
	@echo "Generating 100 test requests to populate metrics..."
	@echo "Fibonacci Service URL: http://localhost:8080/fibonacci?n=10"
	@echo ""
	@echo "NOTE: Make sure port-forwarding is active on port 8080!"
	@echo "      Run 'make fibonacci' in another terminal first"
	@echo ""
	@for /L %%i in (1,1,100) do ( \
		curl -s "http://localhost:8080/fibonacci?n=10" >nul & \
		if %%i==10 echo 10 requests completed... & \
		if %%i==25 echo 25 requests completed... & \
		if %%i==50 echo 50 requests completed... & \
		if %%i==75 echo 75 requests completed... \
	)
	@echo ""
	@echo "100 requests completed!"
	@echo "Metrics should now be visible in Grafana dashboard."
	@echo ""

# Show dashboard information
dashboard:
	@echo ""
	@echo "=========================================="
	@echo "Grafana Dashboard Information"
	@echo "=========================================="
	@echo ""
	@echo "Dashboard JSON file location:"
	@cd
	@echo grafana-dashboard.json
	@echo ""
	@echo "Dashboard includes these metrics:"
	@echo "  - Request Rate (req/s)"
	@echo "  - HTTP Status Codes (200, 400, 429, 500)"
	@echo "  - Request Duration (P99, P95, P50)"
	@echo "  - Active Requests"
	@echo "  - Total Requests"
	@echo "  - Success Rate (%%)"
	@echo "  - Average Response Time"
	@echo "  - Rate Limit Reached Count"
	@echo "  - Response Size Distribution"
	@echo "  - Fibonacci Input Distribution"
	@echo ""
	@echo "To import, run: make import-dashboard"
	@echo ""

# Prometheus data source setup instructions
setup-datasource:
	@echo ""
	@echo "=========================================="
	@echo "Setting up Prometheus Data Source"
	@echo "=========================================="
	@echo ""
	@echo "1. Open Grafana at http://localhost:3000"
	@echo "   Run: make grafana"
	@echo ""
	@echo "2. Login with default credentials:"
	@echo "   Username: admin"
	@echo "   Password: admin"
	@echo ""
	@echo "3. Go to Configuration (gear icon) -> Data Sources"
	@echo ""
	@echo "4. Click 'Add data source'"
	@echo ""
	@echo "5. Select 'Prometheus'"
	@echo ""
	@echo "6. Configure the URL (try these in order):"
	@echo ""
	@echo "   Option 1 - Using Service Name (recommended):"
	@echo "     URL: http://prometheus.default.svc.cluster.local:9090"
	@echo ""
	@echo "   Option 2 - Using Service Name (short):"
	@echo "     URL: http://prometheus:9090"
	@echo ""
	@echo "   Option 3 - Using Cluster IP:"
	@kubectl get svc prometheus -o jsonpath='     URL: http://{.spec.clusterIP}:9090'
	@echo ""
	@echo ""
	@echo "7. Set Access: Server (default)"
	@echo ""
	@echo "8. Click 'Save & Test' - you should see success"
	@echo ""
	@echo "If connection fails, make sure Prometheus is running:"
	@echo "  kubectl get pods -l app=prometheus"
	@echo ""
	@echo "Next: make import-dashboard"
	@echo ""

# Dashboard import instructions
import-dashboard:
	@echo ""
	@echo "=========================================="
	@echo "Importing Fibonacci Dashboard"
	@echo "=========================================="
	@echo ""
	@echo "Option 1: Import via File Upload"
	@echo "  1. In Grafana, click '+' -> 'Import'"
	@echo "  2. Click 'Upload JSON file'"
	@echo "  3. Select: grafana-dashboard.json"
	@echo "  4. Click 'Load'"
	@echo "  5. Select 'Prometheus' data source"
	@echo "  6. Click 'Import'"
	@echo ""
	@echo "Option 2: Import via Copy/Paste"
	@echo "  1. Open grafana-dashboard.json in an editor"
	@echo "  2. Copy the entire JSON content"
	@echo "  3. In Grafana, click '+' -> 'Import'"
	@echo "  4. Paste JSON in the text area"
	@echo "  5. Click 'Load'"
	@echo "  6. Select 'Prometheus' data source"
	@echo "  7. Click 'Import'"
	@echo ""
	@echo "Generate traffic to see metrics:"
	@echo "  make traffic"
	@echo ""

# Clean up port-forwards
clean:
	@echo ""
	@echo "Stopping all kubectl port-forward processes..."
	@taskkill /F /FI "WINDOWTITLE eq Grafana Port-Forward*" 2>nul || echo No Grafana port-forward found
	@taskkill /F /FI "WINDOWTITLE eq Prometheus Port-Forward*" 2>nul || echo No Prometheus port-forward found
	@taskkill /F /FI "WINDOWTITLE eq Fibonacci Port-Forward*" 2>nul || echo No Fibonacci port-forward found
	@echo ""
	@echo "All port-forwards stopped!"
	@echo ""

# Quick start - set up everything
quickstart: check-all
	@echo ""
	@echo "=========================================="
	@echo "Quick Start Guide"
	@echo "=========================================="
	@echo ""
	@echo "Step 1: Start port-forwarding"
	@echo "  Terminal 1: make grafana"
	@echo "  Terminal 2: make prometheus"
	@echo "  Terminal 3: make fibonacci"
	@echo ""
	@echo "Or use background mode (opens new windows):"
	@echo "  make all"
	@echo ""
	@echo "Step 2: Setup Prometheus data source"
	@echo "  make setup-datasource"
	@echo ""
	@echo "Step 3: Import dashboard"
	@echo "  make import-dashboard"
	@echo ""
	@echo "Step 4: Generate traffic"
	@echo "  make traffic"
	@echo ""
	@echo "Step 5: View metrics in Grafana"
	@echo "  http://localhost:3000"
	@echo ""

# Restart Grafana and Prometheus
restart:
	@echo ""
	@echo "=========================================="
	@echo "Restarting Grafana and Prometheus"
	@echo "=========================================="
	@echo ""
	@echo "Restarting Grafana deployment..."
	kubectl rollout restart deployment/grafana
	@echo "Waiting for Grafana to be ready..."
	kubectl rollout status deployment/grafana
	@echo ""
	@echo "Restarting Prometheus deployment..."
	kubectl rollout restart deployment/prometheus
	@echo "Waiting for Prometheus to be ready..."
	kubectl rollout status deployment/prometheus
	@echo ""
	@echo "=========================================="
	@echo "Restart completed!"
	@echo "=========================================="
	@echo ""
	@echo "Next steps:"
	@echo "  1. Port-forward services: make all"
	@echo "  2. Generate traffic: make traffic"
	@echo "  3. Access Grafana: http://localhost:3000"
	@echo ""

# Restart all services including Fibonacci
restart-all:
	@echo ""
	@echo "=========================================="
	@echo "Restarting ALL Services"
	@echo "=========================================="
	@echo ""
	@echo "Restarting Fibonacci deployment..."
	kubectl rollout restart deployment/fibonacci-deployment
	@echo "Waiting for Fibonacci to be ready..."
	kubectl rollout status deployment/fibonacci-deployment
	@echo ""
	@echo "Restarting Grafana deployment..."
	kubectl rollout restart deployment/grafana
	@echo "Waiting for Grafana to be ready..."
	kubectl rollout status deployment/grafana
	@echo ""
	@echo "Restarting Prometheus deployment..."
	kubectl rollout restart deployment/prometheus
	@echo "Waiting for Prometheus to be ready..."
	kubectl rollout status deployment/prometheus
	@echo ""
	@echo "Restarting OpenTelemetry Collector..."
	kubectl rollout restart deployment/otel-collector
	@echo "Waiting for OTel Collector to be ready..."
	kubectl rollout status deployment/otel-collector
	@echo ""
	@echo "=========================================="
	@echo "All services restarted successfully!"
	@echo "=========================================="
	@echo ""
	@echo "All pods are now running fresh instances."
	@echo ""
	@echo "Next steps:"
	@echo "  1. Port-forward services: make all"
	@echo "  2. Generate traffic: make traffic"
	@echo "  3. Access Grafana: http://localhost:3000"
	@echo ""
	@kubectl get pods


