@echo off
REM Script to setup and access Grafana dashboard for Fibonacci service
REM This script helps you quickly access Grafana and verify metrics

echo ========================================
echo Fibonacci Service - Grafana Setup
echo ========================================
echo.

echo Step 1: Checking if Grafana pod is running...
kubectl get pods -l app=grafana
if %errorlevel% neq 0 (
    echo ERROR: Grafana pod not found. Please deploy the application first.
    pause
    exit /b 1
)
echo.

echo Step 2: Checking if Prometheus pod is running...
kubectl get pods -l app=prometheus
if %errorlevel% neq 0 (
    echo ERROR: Prometheus pod not found. Please deploy the application first.
    pause
    exit /b 1
)
echo.

echo Step 3: Checking if Fibonacci service is running...
kubectl get svc fibonacci-service
if %errorlevel% neq 0 (
    echo ERROR: Fibonacci service not found. Please deploy the application first.
    pause
    exit /b 1
)
echo.

echo ========================================
echo All services are running!
echo ========================================
echo.

echo What would you like to do?
echo.
echo 1. Port-forward Grafana (access at http://localhost:3000)
echo 2. Port-forward Prometheus (access at http://localhost:9090)
echo 3. Port-forward Fibonacci Service (access at http://localhost:8080)
echo 4. Port-forward ALL services
echo 5. Generate test traffic (100 requests)
echo 6. View Grafana dashboard JSON location
echo 7. Exit
echo.

set /p choice="Enter your choice (1-7): "

if "%choice%"=="1" goto grafana
if "%choice%"=="2" goto prometheus
if "%choice%"=="3" goto fibonacci
if "%choice%"=="4" goto all
if "%choice%"=="5" goto traffic
if "%choice%"=="6" goto dashboard
if "%choice%"=="7" goto end

:grafana
echo.
echo Starting port-forward for Grafana...
echo Access Grafana at: http://localhost:3000
echo Default credentials: admin / admin
echo.
echo Press Ctrl+C to stop port-forwarding
kubectl port-forward deployment/grafana 3000:3000
goto end

:prometheus
echo.
echo Starting port-forward for Prometheus...
echo Access Prometheus at: http://localhost:9090
echo.
echo Press Ctrl+C to stop port-forwarding
kubectl port-forward svc/prometheus 9090:9090
goto end

:fibonacci
echo.
echo Starting port-forward for Fibonacci Service...
echo Access Fibonacci at: http://localhost:8080
echo Metrics endpoint: http://localhost:8080/metrics
echo.
echo Press Ctrl+C to stop port-forwarding
kubectl port-forward svc/fibonacci-service 8080:8080
goto end

:all
echo.
echo Starting port-forward for ALL services...
echo This will open 3 command windows:
echo   - Grafana: http://localhost:3000
echo   - Prometheus: http://localhost:9090
echo   - Fibonacci: http://localhost:8080
echo.
start "Grafana Port-Forward" cmd /k kubectl port-forward deployment/grafana 3000:3000
timeout /t 2 /nobreak >nul
start "Prometheus Port-Forward" cmd /k kubectl port-forward svc/prometheus 9090:9090
timeout /t 2 /nobreak >nul
start "Fibonacci Port-Forward" cmd /k kubectl port-forward svc/fibonacci-service 8080:8080
echo.
echo All port-forwards started in separate windows!
echo You can now access:
echo   - Grafana: http://localhost:3000 (admin/admin)
echo   - Prometheus: http://localhost:9090
echo   - Fibonacci: http://localhost:8080
echo.
pause
goto end

:traffic
echo.
echo Generating 100 test requests to populate metrics...
echo Fibonacci Service URL: http://localhost:8080/fibonacci?n=10
echo.
echo NOTE: Make sure port-forwarding is active on port 8080!
echo.
set /p confirm="Continue? (Y/N): "
if /i not "%confirm%"=="Y" goto end

for /L %%i in (1,1,100) do (
    curl -s "http://localhost:8080/fibonacci?n=10" >nul
    if %%i==10 echo 10 requests completed...
    if %%i==25 echo 25 requests completed...
    if %%i==50 echo 50 requests completed...
    if %%i==75 echo 75 requests completed...
)
echo.
echo 100 requests completed!
echo Metrics should now be visible in Grafana dashboard.
echo.
pause
goto end

:dashboard
echo.
echo ========================================
echo Grafana Dashboard Information
echo ========================================
echo.
echo Dashboard JSON file location:
echo %CD%\grafana-dashboard.json
echo.
echo To import in Grafana:
echo 1. Open Grafana at http://localhost:3000
echo 2. Login with admin/admin
echo 3. Go to Configuration ^> Data Sources
echo 4. Add Prometheus data source: http://prometheus:9090
echo 5. Go to Dashboards ^> Import
echo 6. Upload grafana-dashboard.json
echo.
echo Dashboard includes these metrics:
echo   - Request Rate (req/s)
echo   - HTTP Status Codes (200, 400, 429, 500)
echo   - Request Duration (P99, P95, P50)
echo   - Active Requests
echo   - Total Requests
echo   - Success Rate (%)
echo   - Average Response Time
echo   - Rate Limit Reached Count
echo   - Response Size Distribution
echo   - Fibonacci Input Distribution
echo.
pause
goto end

:end
echo.
echo Exiting...
exit /b 0

