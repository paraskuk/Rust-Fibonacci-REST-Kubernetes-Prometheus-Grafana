# Authentication & Authorization Testing Scripts

This document provides comprehensive testing scripts and validation procedures to verify the authentication and authorization implementation is working correctly.

## Test Script Collection

### 1. RBAC Permissions Test Script

Create `test-rbac.sh`:

```bash
#!/bin/bash

echo "=== RBAC Permissions Testing ==="
echo "Testing various user permissions..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to test permission
test_permission() {
    local user=$1
    local verb=$2
    local resource=$3
    local namespace=$4
    
    if [ -n "$namespace" ]; then
        result=$(kubectl auth can-i $verb $resource --as=$user -n $namespace 2>/dev/null)
        echo -n "Testing: $user can $verb $resource in namespace $namespace: "
    else
        result=$(kubectl auth can-i $verb $resource --as=$user 2>/dev/null)
        echo -n "Testing: $user can $verb $resource (cluster-wide): "
    fi
    
    if [ "$result" = "yes" ]; then
        echo -e "${GREEN}✓ ALLOWED${NC}"
        return 0
    else
        echo -e "${RED}✗ DENIED${NC}"
        return 1
    fi
}

echo -e "\n${YELLOW}1. Testing Admin Permissions (should all be ALLOWED)${NC}"
test_permission "admin" "create" "deployments" "fibonacci-app"
test_permission "admin" "delete" "pods" "fibonacci-app"
test_permission "admin" "create" "namespaces"
test_permission "admin" "get" "secrets" "monitoring"
test_permission "admin" "create" "clusterroles"

echo -e "\n${YELLOW}2. Testing Reader Permissions${NC}"
echo "Should be ALLOWED:"
test_permission "reader" "get" "pods" "fibonacci-app"
test_permission "reader" "list" "deployments" "fibonacci-app"
test_permission "reader" "watch" "services" "monitoring"
test_permission "reader" "get" "pods/log" "fibonacci-app"

echo "Should be DENIED:"
test_permission "reader" "create" "deployments" "fibonacci-app"
test_permission "reader" "delete" "pods" "fibonacci-app"
test_permission "reader" "create" "secrets" "fibonacci-app"
test_permission "reader" "create" "namespaces"

echo -e "\n${YELLOW}3. Testing Service Account Permissions${NC}"
test_permission "system:serviceaccount:fibonacci-app:fibonacci-service-account" "get" "pods" "fibonacci-app"
test_permission "system:serviceaccount:fibonacci-app:fibonacci-service-account" "create" "deployments" "fibonacci-app"
test_permission "system:serviceaccount:monitoring:prometheus-service-account" "get" "pods"

echo -e "\n${YELLOW}4. Testing Cross-Namespace Access${NC}"
test_permission "reader" "get" "pods" "monitoring"
test_permission "system:serviceaccount:fibonacci-app:fibonacci-service-account" "get" "pods" "monitoring"

echo -e "\n=== RBAC Testing Complete ==="
```

### 2. Authentication Test Script

Create `test-authentication.sh`:

```bash
#!/bin/bash

echo "=== Authentication Testing ==="

# Configuration
FIBONACCI_URL="http://localhost:8080"
ADMIN_TOKEN=""
READER_TOKEN=""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Function to generate test JWT token (for testing purposes only)
generate_test_token() {
    local role=$1
    local secret=${JWT_SECRET:-"your-secret-key"}
    
    # This is a simple JWT generation for testing
    # In production, use proper JWT libraries
    header='{"alg":"HS256","typ":"JWT"}'
    payload="{\"sub\":\"test-user\",\"role\":\"$role\",\"exp\":$(($(date +%s) + 3600))}"
    
    header_b64=$(echo -n "$header" | base64 | tr -d '=' | tr '/+' '_-' | tr -d '\n')
    payload_b64=$(echo -n "$payload" | base64 | tr -d '=' | tr '/+' '_-' | tr -d '\n')
    
    signature=$(echo -n "${header_b64}.${payload_b64}" | openssl dgst -sha256 -hmac "$secret" -binary | base64 | tr -d '=' | tr '/+' '_-' | tr -d '\n')
    
    echo "${header_b64}.${payload_b64}.${signature}"
}

# Test unauthenticated access
test_unauthenticated() {
    echo -e "\n${YELLOW}Testing Unauthenticated Access${NC}"
    
    echo -n "Testing /health endpoint (should be accessible): "
    response=$(curl -s -o /dev/null -w "%{http_code}" "$FIBONACCI_URL/health")
    if [ "$response" = "200" ]; then
        echo -e "${GREEN}✓ SUCCESS (200)${NC}"
    else
        echo -e "${RED}✗ FAILED ($response)${NC}"
    fi
    
    echo -n "Testing /api/fibonacci/10 without token (should be 401): "
    response=$(curl -s -o /dev/null -w "%{http_code}" "$FIBONACCI_URL/api/fibonacci/10")
    if [ "$response" = "401" ]; then
        echo -e "${GREEN}✓ CORRECTLY DENIED (401)${NC}"
    else
        echo -e "${RED}✗ UNEXPECTED RESPONSE ($response)${NC}"
    fi
}

# Test authenticated access
test_authenticated() {
    echo -e "\n${YELLOW}Testing Authenticated Access${NC}"
    
    # Generate test tokens
    ADMIN_TOKEN=$(generate_test_token "admin")
    READER_TOKEN=$(generate_test_token "reader")
    
    echo -n "Testing admin access to /api/fibonacci/10: "
    response=$(curl -s -o /dev/null -w "%{http_code}" -H "Authorization: Bearer $ADMIN_TOKEN" "$FIBONACCI_URL/api/fibonacci/10")
    if [ "$response" = "200" ]; then
        echo -e "${GREEN}✓ SUCCESS (200)${NC}"
    else
        echo -e "${RED}✗ FAILED ($response)${NC}"
    fi
    
    echo -n "Testing reader access to /api/fibonacci/10: "
    response=$(curl -s -o /dev/null -w "%{http_code}" -H "Authorization: Bearer $READER_TOKEN" "$FIBONACCI_URL/api/fibonacci/10")
    if [ "$response" = "200" ]; then
        echo -e "${GREEN}✓ SUCCESS (200)${NC}"
    else
        echo -e "${RED}✗ FAILED ($response)${NC}"
    fi
    
    echo -n "Testing admin access to /api/admin: "
    response=$(curl -s -o /dev/null -w "%{http_code}" -H "Authorization: Bearer $ADMIN_TOKEN" "$FIBONACCI_URL/api/admin")
    if [ "$response" = "200" ]; then
        echo -e "${GREEN}✓ SUCCESS (200)${NC}"
    else
        echo -e "${RED}✗ FAILED ($response)${NC}"
    fi
    
    echo -n "Testing reader access to /api/admin (should be 403): "
    response=$(curl -s -o /dev/null -w "%{http_code}" -H "Authorization: Bearer $READER_TOKEN" "$FIBONACCI_URL/api/admin")
    if [ "$response" = "403" ]; then
        echo -e "${GREEN}✓ CORRECTLY DENIED (403)${NC}"
    else
        echo -e "${RED}✗ UNEXPECTED RESPONSE ($response)${NC}"
    fi
}

# Test invalid tokens
test_invalid_tokens() {
    echo -e "\n${YELLOW}Testing Invalid Tokens${NC}"
    
    echo -n "Testing with invalid token: "
    response=$(curl -s -o /dev/null -w "%{http_code}" -H "Authorization: Bearer invalid.token.here" "$FIBONACCI_URL/api/fibonacci/10")
    if [ "$response" = "401" ]; then
        echo -e "${GREEN}✓ CORRECTLY DENIED (401)${NC}"
    else
        echo -e "${RED}✗ UNEXPECTED RESPONSE ($response)${NC}"
    fi
    
    echo -n "Testing with expired token: "
    # Generate expired token (exp in the past)
    header='{"alg":"HS256","typ":"JWT"}'
    payload="{\"sub\":\"test-user\",\"role\":\"admin\",\"exp\":$(($(date +%s) - 3600))}"
    header_b64=$(echo -n "$header" | base64 | tr -d '=' | tr '/+' '_-' | tr -d '\n')
    payload_b64=$(echo -n "$payload" | base64 | tr -d '=' | tr '/+' '_-' | tr -d '\n')
    signature=$(echo -n "${header_b64}.${payload_b64}" | openssl dgst -sha256 -hmac "your-secret-key" -binary | base64 | tr -d '=' | tr '/+' '_-' | tr -d '\n')
    expired_token="${header_b64}.${payload_b64}.${signature}"
    
    response=$(curl -s -o /dev/null -w "%{http_code}" -H "Authorization: Bearer $expired_token" "$FIBONACCI_URL/api/fibonacci/10")
    if [ "$response" = "401" ]; then
        echo -e "${GREEN}✓ CORRECTLY DENIED (401)${NC}"
    else
        echo -e "${RED}✗ UNEXPECTED RESPONSE ($response)${NC}"
    fi
}

# Port forward the service for testing
echo "Starting port-forward to fibonacci service..."
kubectl port-forward svc/fibonacci-service 8080:8080 -n fibonacci-app &
PORT_FORWARD_PID=$!
sleep 3

# Run tests
test_unauthenticated
test_authenticated
test_invalid_tokens

# Cleanup
kill $PORT_FORWARD_PID 2>/dev/null

echo -e "\n=== Authentication Testing Complete ==="
```

### 3. Network Policy Test Script

Create `test-network-policies.sh`:

```bash
#!/bin/bash

echo "=== Network Policy Testing ==="

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Test network connectivity
test_network_connectivity() {
    local from_pod=$1
    local from_namespace=$2
    local to_service=$3
    local to_namespace=$4
    local port=$5
    local expected=$6
    
    echo -n "Testing $from_pod -> $to_service:$port (expect $expected): "
    
    # Run connectivity test
    result=$(kubectl exec -n $from_namespace $from_pod -- nc -zv $to_service.$to_namespace.svc.cluster.local $port 2>&1)
    
    if [[ $result == *"open"* ]] && [ "$expected" = "ALLOWED" ]; then
        echo -e "${GREEN}✓ ALLOWED${NC}"
        return 0
    elif [[ $result != *"open"* ]] && [ "$expected" = "DENIED" ]; then
        echo -e "${GREEN}✓ CORRECTLY DENIED${NC}"
        return 0
    else
        echo -e "${RED}✗ UNEXPECTED RESULT${NC}"
        echo "  Result: $result"
        return 1
    fi
}

# Create test pods
create_test_pods() {
    echo "Creating test pods..."
    
    # Test pod in fibonacci-app namespace
    kubectl run test-pod-app --image=busybox --restart=Never -n fibonacci-app -- sleep 3600 2>/dev/null || true
    
    # Test pod in monitoring namespace
    kubectl run test-pod-monitoring --image=busybox --restart=Never -n monitoring -- sleep 3600 2>/dev/null || true
    
    # Test pod in default namespace
    kubectl run test-pod-default --image=busybox --restart=Never -n default -- sleep 3600 2>/dev/null || true
    
    # Wait for pods to be ready
    echo "Waiting for test pods to be ready..."
    kubectl wait --for=condition=Ready pod/test-pod-app -n fibonacci-app --timeout=60s
    kubectl wait --for=condition=Ready pod/test-pod-monitoring -n monitoring --timeout=60s
    kubectl wait --for=condition=Ready pod/test-pod-default -n default --timeout=60s
}

# Run network policy tests
run_network_tests() {
    echo -e "\n${YELLOW}Testing Network Policies${NC}"
    
    # Test access to fibonacci service from different namespaces
    echo -e "\n${YELLOW}Testing access to Fibonacci service (port 8080)${NC}"
    test_network_connectivity "test-pod-app" "fibonacci-app" "fibonacci-service" "fibonacci-app" "8080" "ALLOWED"
    test_network_connectivity "test-pod-monitoring" "monitoring" "fibonacci-service" "fibonacci-app" "8080" "ALLOWED"
    test_network_connectivity "test-pod-default" "default" "fibonacci-service" "fibonacci-app" "8080" "DENIED"
    
    # Test access to prometheus service
    echo -e "\n${YELLOW}Testing access to Prometheus service (port 9090)${NC}"
    test_network_connectivity "test-pod-app" "fibonacci-app" "prometheus" "monitoring" "9090" "DENIED"
    test_network_connectivity "test-pod-monitoring" "monitoring" "prometheus" "monitoring" "9090" "ALLOWED"
    test_network_connectivity "test-pod-default" "default" "prometheus" "monitoring" "9090" "DENIED"
    
    # Test access to otel-collector
    echo -e "\n${YELLOW}Testing access to OpenTelemetry Collector (port 8889)${NC}"
    test_network_connectivity "test-pod-app" "fibonacci-app" "otel-collector" "monitoring" "8889" "ALLOWED"
    test_network_connectivity "test-pod-monitoring" "monitoring" "otel-collector" "monitoring" "8889" "ALLOWED"
    test_network_connectivity "test-pod-default" "default" "otel-collector" "monitoring" "8889" "DENIED"
    
    # Test DNS resolution (should work from all pods)
    echo -e "\n${YELLOW}Testing DNS resolution${NC}"
    echo -n "Testing DNS from fibonacci-app namespace: "
    if kubectl exec -n fibonacci-app test-pod-app -- nslookup kubernetes.default.svc.cluster.local >/dev/null 2>&1; then
        echo -e "${GREEN}✓ SUCCESS${NC}"
    else
        echo -e "${RED}✗ FAILED${NC}"
    fi
    
    echo -n "Testing DNS from monitoring namespace: "
    if kubectl exec -n monitoring test-pod-monitoring -- nslookup kubernetes.default.svc.cluster.local >/dev/null 2>&1; then
        echo -e "${GREEN}✓ SUCCESS${NC}"
    else
        echo -e "${RED}✗ FAILED${NC}"
    fi
}

# Cleanup test pods
cleanup_test_pods() {
    echo -e "\n${YELLOW}Cleaning up test pods...${NC}"
    kubectl delete pod test-pod-app -n fibonacci-app --ignore-not-found=true
    kubectl delete pod test-pod-monitoring -n monitoring --ignore-not-found=true
    kubectl delete pod test-pod-default -n default --ignore-not-found=true
}

# Main execution
create_test_pods
run_network_tests
cleanup_test_pods

echo -e "\n=== Network Policy Testing Complete ==="
```

### 4. Security Validation Script

Create `validate-security.sh`:

```bash
#!/bin/bash

echo "=== Security Validation ==="

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check for security issues
check_security_contexts() {
    echo -e "\n${YELLOW}Checking Pod Security Contexts${NC}"
    
    echo -n "Checking for pods running as root: "
    root_pods=$(kubectl get pods -A -o jsonpath='{range .items[*]}{.metadata.namespace}/{.metadata.name}: {.spec.securityContext.runAsUser}{"\n"}{end}' | grep ': 0$' | wc -l)
    if [ "$root_pods" -eq 0 ]; then
        echo -e "${GREEN}✓ No pods running as root${NC}"
    else
        echo -e "${RED}✗ Found $root_pods pods running as root${NC}"
    fi
    
    echo -n "Checking for privileged containers: "
    privileged_containers=$(kubectl get pods -A -o jsonpath='{range .items[*]}{range .spec.containers[*]}{.securityContext.privileged}{"\n"}{end}{end}' | grep true | wc -l)
    if [ "$privileged_containers" -eq 0 ]; then
        echo -e "${GREEN}✓ No privileged containers found${NC}"
    else
        echo -e "${RED}✗ Found $privileged_containers privileged containers${NC}"
    fi
    
    echo -n "Checking for containers with privilege escalation: "
    privilege_escalation=$(kubectl get pods -A -o jsonpath='{range .items[*]}{range .spec.containers[*]}{.securityContext.allowPrivilegeEscalation}{"\n"}{end}{end}' | grep true | wc -l)
    if [ "$privilege_escalation" -eq 0 ]; then
        echo -e "${GREEN}✓ No containers with privilege escalation${NC}"
    else
        echo -e "${RED}✗ Found $privilege_escalation containers with privilege escalation${NC}"
    fi
}

# Check RBAC configuration
check_rbac_config() {
    echo -e "\n${YELLOW}Checking RBAC Configuration${NC}"
    
    echo -n "Checking for ClusterRoles: "
    cluster_roles=$(kubectl get clusterroles | grep fibonacci | wc -l)
    if [ "$cluster_roles" -gt 0 ]; then
        echo -e "${GREEN}✓ Found $cluster_roles fibonacci ClusterRoles${NC}"
    else
        echo -e "${RED}✗ No fibonacci ClusterRoles found${NC}"
    fi
    
    echo -n "Checking for ClusterRoleBindings: "
    cluster_role_bindings=$(kubectl get clusterrolebindings | grep fibonacci | wc -l)
    if [ "$cluster_role_bindings" -gt 0 ]; then
        echo -e "${GREEN}✓ Found $cluster_role_bindings fibonacci ClusterRoleBindings${NC}"
    else
        echo -e "${RED}✗ No fibonacci ClusterRoleBindings found${NC}"
    fi
    
    echo -n "Checking for namespace-specific Roles: "
    roles=$(kubectl get roles -A | grep fibonacci | wc -l)
    if [ "$roles" -gt 0 ]; then
        echo -e "${GREEN}✓ Found $roles fibonacci Roles${NC}"
    else
        echo -e "${YELLOW}⚠ No fibonacci Roles found${NC}"
    fi
    
    echo -n "Checking for RoleBindings: "
    role_bindings=$(kubectl get rolebindings -A | grep fibonacci | wc -l)
    if [ "$role_bindings" -gt 0 ]; then
        echo -e "${GREEN}✓ Found $role_bindings fibonacci RoleBindings${NC}"
    else
        echo -e "${YELLOW}⚠ No fibonacci RoleBindings found${NC}"
    fi
}

# Check network policies
check_network_policies() {
    echo -e "\n${YELLOW}Checking Network Policies${NC}"
    
    echo -n "Checking for NetworkPolicies in fibonacci-app namespace: "
    app_netpols=$(kubectl get networkpolicies -n fibonacci-app 2>/dev/null | grep -v NAME | wc -l)
    if [ "$app_netpols" -gt 0 ]; then
        echo -e "${GREEN}✓ Found $app_netpols NetworkPolicies${NC}"
    else
        echo -e "${YELLOW}⚠ No NetworkPolicies found in fibonacci-app namespace${NC}"
    fi
    
    echo -n "Checking for NetworkPolicies in monitoring namespace: "
    monitoring_netpols=$(kubectl get networkpolicies -n monitoring 2>/dev/null | grep -v NAME | wc -l)
    if [ "$monitoring_netpols" -gt 0 ]; then
        echo -e "${GREEN}✓ Found $monitoring_netpols NetworkPolicies${NC}"
    else
        echo -e "${YELLOW}⚠ No NetworkPolicies found in monitoring namespace${NC}"
    fi
}

# Check secrets
check_secrets() {
    echo -e "\n${YELLOW}Checking Secrets${NC}"
    
    echo -n "Checking for JWT secrets: "
    jwt_secrets=$(kubectl get secrets -A | grep jwt | wc -l)
    if [ "$jwt_secrets" -gt 0 ]; then
        echo -e "${GREEN}✓ Found $jwt_secrets JWT secrets${NC}"
    else
        echo -e "${YELLOW}⚠ No JWT secrets found${NC}"
    fi
    
    echo -n "Checking for service account tokens: "
    sa_tokens=$(kubectl get secrets -A | grep token | wc -l)
    if [ "$sa_tokens" -gt 0 ]; then
        echo -e "${GREEN}✓ Found $sa_tokens service account tokens${NC}"
    else
        echo -e "${RED}✗ No service account tokens found${NC}"
    fi
}

# Check service accounts
check_service_accounts() {
    echo -e "\n${YELLOW}Checking Service Accounts${NC}"
    
    echo -n "Checking for fibonacci service accounts: "
    fibonacci_sas=$(kubectl get serviceaccounts -A | grep fibonacci | wc -l)
    if [ "$fibonacci_sas" -gt 0 ]; then
        echo -e "${GREEN}✓ Found $fibonacci_sas fibonacci service accounts${NC}"
    else
        echo -e "${RED}✗ No fibonacci service accounts found${NC}"
    fi
    
    echo -n "Checking for monitoring service accounts: "
    monitoring_sas=$(kubectl get serviceaccounts -n monitoring | grep -E "(prometheus|otel|grafana)" | wc -l)
    if [ "$monitoring_sas" -gt 0 ]; then
        echo -e "${GREEN}✓ Found $monitoring_sas monitoring service accounts${NC}"
    else
        echo -e "${YELLOW}⚠ No monitoring service accounts found${NC}"
    fi
}

# Generate security report
generate_security_report() {
    echo -e "\n${YELLOW}Generating Security Report${NC}"
    
    report_file="security-report-$(date +%Y%m%d-%H%M%S).txt"
    
    {
        echo "Kubernetes Security Report"
        echo "Generated: $(date)"
        echo "=========================="
        echo
        
        echo "RBAC Resources:"
        kubectl get clusterroles,clusterrolebindings,roles,rolebindings -A | grep fibonacci
        echo
        
        echo "Service Accounts:"
        kubectl get serviceaccounts -A | grep -E "(fibonacci|prometheus|otel|grafana)"
        echo
        
        echo "Network Policies:"
        kubectl get networkpolicies -A
        echo
        
        echo "Secrets:"
        kubectl get secrets -A | grep -E "(jwt|token|auth)"
        echo
        
        echo "Pod Security Contexts:"
        kubectl get pods -A -o custom-columns="NAMESPACE:.metadata.namespace,NAME:.metadata.name,USER:.spec.securityContext.runAsUser,PRIVILEGED:.spec.containers[*].securityContext.privileged"
        
    } > "$report_file"
    
    echo "Security report saved to: $report_file"
}

# Main execution
check_security_contexts
check_rbac_config
check_network_policies
check_secrets
check_service_accounts
generate_security_report

echo -e "\n=== Security Validation Complete ==="
```

### 5. Complete Test Suite Runner

Create `run-all-tests.sh`:

```bash
#!/bin/bash

echo "========================================="
echo "   Kubernetes Auth/Authz Test Suite"
echo "========================================="

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test results tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run a test script
run_test() {
    local test_name=$1
    local test_script=$2
    
    echo -e "\n${BLUE}🧪 Running: $test_name${NC}"
    echo "=" | perl -lane 'print "=" x 50'
    
    if [ -f "$test_script" ] && [ -x "$test_script" ]; then
        if ./"$test_script"; then
            echo -e "${GREEN}✅ $test_name: PASSED${NC}"
            ((PASSED_TESTS++))
        else
            echo -e "${RED}❌ $test_name: FAILED${NC}"
            ((FAILED_TESTS++))
        fi
    else
        echo -e "${YELLOW}⚠️  $test_name: SKIPPED (script not found or not executable)${NC}"
    fi
    
    ((TOTAL_TESTS++))
}

# Pre-flight checks
preflight_checks() {
    echo -e "${BLUE}🔍 Running Pre-flight Checks${NC}"
    echo "=" | perl -lane 'print "=" x 50'
    
    # Check kubectl
    if ! command -v kubectl &> /dev/null; then
        echo -e "${RED}❌ kubectl not found${NC}"
        exit 1
    fi
    
    # Check cluster connectivity
    if ! kubectl cluster-info &> /dev/null; then
        echo -e "${RED}❌ Cannot connect to Kubernetes cluster${NC}"
        exit 1
    fi
    
    # Check namespaces
    echo -n "Checking fibonacci-app namespace: "
    if kubectl get namespace fibonacci-app &> /dev/null; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${YELLOW}⚠️  Not found${NC}"
    fi
    
    echo -n "Checking monitoring namespace: "
    if kubectl get namespace monitoring &> /dev/null; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${YELLOW}⚠️  Not found${NC}"
    fi
    
    echo -e "${GREEN}✅ Pre-flight checks completed${NC}"
}

# Make test scripts executable
chmod +x test-rbac.sh 2>/dev/null
chmod +x test-authentication.sh 2>/dev/null
chmod +x test-network-policies.sh 2>/dev/null
chmod +x validate-security.sh 2>/dev/null

# Run all tests
preflight_checks

run_test "RBAC Permissions Test" "test-rbac.sh"
run_test "Authentication Test" "test-authentication.sh"
run_test "Network Policies Test" "test-network-policies.sh"
run_test "Security Validation" "validate-security.sh"

# Generate summary report
echo -e "\n${BLUE}📊 Test Summary Report${NC}"
echo "=" | perl -lane 'print "=" x 50'
echo -e "Total Tests: $TOTAL_TESTS"
echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
echo -e "${RED}Failed: $FAILED_TESTS${NC}"

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All tests passed! Authentication and authorization setup is working correctly.${NC}"
    exit 0
else
    echo -e "\n${RED}⚠️  Some tests failed. Please review the output above and fix any issues.${NC}"
    exit 1
fi
```

## Usage Instructions

1. **Make scripts executable:**
   ```bash
   chmod +x *.sh
   ```

2. **Run individual tests:**
   ```bash
   ./test-rbac.sh
   ./test-authentication.sh
   ./test-network-policies.sh
   ./validate-security.sh
   ```

3. **Run complete test suite:**
   ```bash
   ./run-all-tests.sh
   ```

4. **Set environment variables for authentication tests:**
   ```bash
   export JWT_SECRET="your-jwt-secret-key"
   export FIBONACCI_URL="http://your-fibonacci-service"
   ```

## Expected Results

### RBAC Tests
- Admin users should have full access to all resources
- Reader users should only have read access to specified resources
- Service accounts should have appropriate namespace-scoped permissions

### Authentication Tests
- Unauthenticated requests to protected endpoints should return 401
- Valid tokens should allow access based on role
- Invalid/expired tokens should be rejected

### Network Policy Tests
- Traffic should be allowed only according to defined policies
- Cross-namespace communication should be restricted appropriately
- DNS resolution should work from all pods

### Security Validation
- No containers should run as root
- No privileged containers should exist
- All required RBAC resources should be present
- Network policies should be in place

These test scripts provide comprehensive validation of your authentication and authorization implementation.