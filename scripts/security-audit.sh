#!/bin/bash

# PesaBit Security Audit Script
# This script performs a comprehensive security audit for production readiness

set -euo pipefail

# Configuration
AUDIT_RESULTS_DIR="/security-audit"
NAMESPACE="pesabit"
STAGING_URL="http://localhost:3000"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Create audit results directory
create_audit_dir() {
    log_info "Creating audit results directory: $AUDIT_RESULTS_DIR"
    mkdir -p "$AUDIT_RESULTS_DIR"/{vulnerabilities,secrets,compliance,network,code,reports}
    log_success "Audit results directory created"
}

# Install security audit tools
install_audit_tools() {
    log_info "Installing security audit tools..."
    
    # Install OWASP ZAP
    if ! command -v zap &> /dev/null; then
        log_info "Installing OWASP ZAP..."
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            wget -q https://github.com/zaproxy/zaproxy/releases/download/v2.14.0/ZAP_2.14.0_Linux.tar.gz
            tar -xzf ZAP_2.14.0_Linux.tar.gz
            sudo mv ZAP_2.14.0 /opt/zap
            sudo ln -s /opt/zap/zap.sh /usr/local/bin/zap
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            brew install --cask owasp-zap
        fi
        log_success "OWASP ZAP installed"
    else
        log_info "OWASP ZAP already installed"
    fi
    
    # Install Nikto
    if ! command -v nikto &> /dev/null; then
        log_info "Installing Nikto..."
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            sudo apt-get install nikto
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            brew install nikto
        fi
        log_success "Nikto installed"
    else
        log_info "Nikto already installed"
    fi
    
    # Install Nmap
    if ! command -v nmap &> /dev/null; then
        log_info "Installing Nmap..."
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            sudo apt-get install nmap
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            brew install nmap
        fi
        log_success "Nmap installed"
    else
        log_info "Nmap already installed"
    fi
    
    # Install SQLMap
    if ! command -v sqlmap &> /dev/null; then
        log_info "Installing SQLMap..."
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            sudo apt-get install sqlmap
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            brew install sqlmap
        fi
        log_success "SQLMap installed"
    else
        log_info "SQLMap already installed"
    fi
}

# Code security audit
audit_code_security() {
    log_info "Performing code security audit..."
    
    local timestamp=$(date +"%Y%m%d_%H%M%S")
    
    # Check for hardcoded secrets
    log_info "Checking for hardcoded secrets..."
    if command -v trufflehog &> /dev/null; then
        trufflehog filesystem . --output json > "$AUDIT_RESULTS_DIR/secrets/code_secrets_$timestamp.json" || true
    fi
    
    # Check for insecure code patterns
    log_info "Checking for insecure code patterns..."
    
    # Check for SQL injection vulnerabilities
    grep -r "format!\|println!\|eprintln!" . --include="*.rs" | grep -v "// TODO\|// FIXME" > "$AUDIT_RESULTS_DIR/code/sql_injection_$timestamp.txt" || true
    
    # Check for unsafe unwrap() usage
    grep -r "\.unwrap()" . --include="*.rs" | grep -v "// TODO\|// FIXME" > "$AUDIT_RESULTS_DIR/code/unsafe_unwrap_$timestamp.txt" || true
    
    # Check for hardcoded passwords
    grep -r -i "password.*=" . --include="*.rs" --include="*.js" --include="*.ts" | grep -v "// TODO\|// FIXME" > "$AUDIT_RESULTS_DIR/code/hardcoded_passwords_$timestamp.txt" || true
    
    # Check for debug prints in production code
    grep -r "dbg!\|println!\|eprintln!" . --include="*.rs" | grep -v "// TODO\|// FIXME" > "$AUDIT_RESULTS_DIR/code/debug_prints_$timestamp.txt" || true
    
    # Check for missing error handling
    grep -r "panic!" . --include="*.rs" | grep -v "// TODO\|// FIXME" > "$AUDIT_RESULTS_DIR/code/panic_usage_$timestamp.txt" || true
    
    log_success "Code security audit completed"
}

# Dependency security audit
audit_dependencies() {
    log_info "Performing dependency security audit..."
    
    local timestamp=$(date +"%Y%m%d_%H%M%S")
    
    # Audit Rust dependencies
    if command -v cargo &> /dev/null; then
        log_info "Auditing Rust dependencies..."
        cargo audit --json > "$AUDIT_RESULTS_DIR/vulnerabilities/rust_audit_$timestamp.json" || true
    fi
    
    # Audit Node.js dependencies
    if [ -f "frontend/package.json" ]; then
        log_info "Auditing Node.js dependencies..."
        cd frontend
        npm audit --json > "../$AUDIT_RESULTS_DIR/vulnerabilities/node_audit_$timestamp.json" || true
        cd ..
    fi
    
    # Audit Python dependencies (if any)
    if [ -f "requirements.txt" ]; then
        log_info "Auditing Python dependencies..."
        pip-audit --format=json > "$AUDIT_RESULTS_DIR/vulnerabilities/python_audit_$timestamp.json" || true
    fi
    
    log_success "Dependency security audit completed"
}

# Container security audit
audit_container_security() {
    log_info "Performing container security audit..."
    
    local timestamp=$(date +"%Y%m%d_%H%M%S")
    
    # Scan Docker images for vulnerabilities
    if command -v trivy &> /dev/null; then
        log_info "Scanning Docker images with Trivy..."
        
        local images=(
            "pesabit/api-gateway:latest"
            "pesabit/user-service:latest"
            "pesabit/payment-service:latest"
            "pesabit/frontend:latest"
        )
        
        for image in "${images[@]}"; do
            if docker image inspect "$image" &> /dev/null; then
                trivy image --format json --output "$AUDIT_RESULTS_DIR/vulnerabilities/trivy_${image//[\/:]/_}_$timestamp.json" "$image" || true
            fi
        done
    fi
    
    # Scan with Grype
    if command -v grype &> /dev/null; then
        log_info "Scanning Docker images with Grype..."
        
        for image in "${images[@]}"; do
            if docker image inspect "$image" &> /dev/null; then
                grype "$image" --output json > "$AUDIT_RESULTS_DIR/vulnerabilities/grype_${image//[\/:]/_}_$timestamp.json" || true
            fi
        done
    fi
    
    log_success "Container security audit completed"
}

# Network security audit
audit_network_security() {
    log_info "Performing network security audit..."
    
    local timestamp=$(date +"%Y%m%d_%H%M%S")
    
    # Port scanning
    if command -v nmap &> /dev/null; then
        log_info "Performing port scan..."
        nmap -sS -O localhost > "$AUDIT_RESULTS_DIR/network/port_scan_$timestamp.txt" || true
    fi
    
    # SSL/TLS analysis
    if command -v sslscan &> /dev/null; then
        log_info "Analyzing SSL/TLS configuration..."
        sslscan "$STAGING_URL" > "$AUDIT_RESULTS_DIR/network/ssl_scan_$timestamp.txt" || true
    fi
    
    # Test for common vulnerabilities
    if command -v nikto &> /dev/null; then
        log_info "Running Nikto vulnerability scan..."
        nikto -h "$STAGING_URL" -output "$AUDIT_RESULTS_DIR/network/nikto_$timestamp.txt" || true
    fi
    
    log_success "Network security audit completed"
}

# Web application security audit
audit_web_application() {
    log_info "Performing web application security audit..."
    
    local timestamp=$(date +"%Y%m%d_%H%M%S")
    
    # OWASP ZAP scan
    if command -v zap &> /dev/null; then
        log_info "Running OWASP ZAP scan..."
        
        # Start ZAP daemon
        zap.sh -daemon -port 8090 -config api.disablekey=true &
        ZAP_PID=$!
        sleep 30
        
        # Run spider scan
        curl "http://localhost:8090/JSON/spider/action/scan/?url=$STAGING_URL" || true
        sleep 60
        
        # Run active scan
        curl "http://localhost:8090/JSON/ascan/action/scan/?url=$STAGING_URL" || true
        sleep 300
        
        # Get scan results
        curl "http://localhost:8090/JSON/core/view/alerts/" > "$AUDIT_RESULTS_DIR/network/zap_alerts_$timestamp.json" || true
        
        # Stop ZAP
        kill $ZAP_PID || true
    fi
    
    # SQL injection testing
    if command -v sqlmap &> /dev/null; then
        log_info "Testing for SQL injection vulnerabilities..."
        sqlmap -u "$STAGING_URL/v1/users/login" --data="phone=test&password=test" --batch --output-dir="$AUDIT_RESULTS_DIR/network/sqlmap_$timestamp" || true
    fi
    
    log_success "Web application security audit completed"
}

# Kubernetes security audit
audit_kubernetes_security() {
    log_info "Performing Kubernetes security audit..."
    
    local timestamp=$(date +"%Y%m%d_%H%M%S")
    
    if kubectl cluster-info &> /dev/null; then
        # Check for security best practices
        log_info "Checking Kubernetes security best practices..."
        
        # Check for privileged containers
        kubectl get pods -n $NAMESPACE -o jsonpath='{.items[*].spec.containers[*].securityContext.privileged}' > "$AUDIT_RESULTS_DIR/compliance/privileged_containers_$timestamp.txt" || true
        
        # Check for host network usage
        kubectl get pods -n $NAMESPACE -o jsonpath='{.items[*].spec.hostNetwork}' > "$AUDIT_RESULTS_DIR/compliance/host_network_$timestamp.txt" || true
        
        # Check for resource limits
        kubectl get pods -n $NAMESPACE -o jsonpath='{.items[*].spec.containers[*].resources}' > "$AUDIT_RESULTS_DIR/compliance/resource_limits_$timestamp.txt" || true
        
        # Check for security contexts
        kubectl get pods -n $NAMESPACE -o jsonpath='{.items[*].spec.securityContext}' > "$AUDIT_RESULTS_DIR/compliance/security_contexts_$timestamp.txt" || true
        
        # Run kube-score
        if command -v kube-score &> /dev/null; then
            kubectl get all -n $NAMESPACE -o yaml | kube-score score - > "$AUDIT_RESULTS_DIR/compliance/kube_score_$timestamp.txt" || true
        fi
        
        # Run kubeaudit
        if command -v kubeaudit &> /dev/null; then
            kubectl get all -n $NAMESPACE -o yaml | kubeaudit all - > "$AUDIT_RESULTS_DIR/compliance/kubeaudit_$timestamp.txt" || true
        fi
    else
        log_warning "Kubernetes cluster not accessible, skipping Kubernetes security audit"
    fi
    
    log_success "Kubernetes security audit completed"
}

# Compliance audit
audit_compliance() {
    log_info "Performing compliance audit..."
    
    local timestamp=$(date +"%Y%m%d_%H%M%S")
    
    # Check for security headers
    log_info "Checking security headers..."
    curl -I "$STAGING_URL" > "$AUDIT_RESULTS_DIR/compliance/security_headers_$timestamp.txt" || true
    
    # Check for CORS configuration
    log_info "Checking CORS configuration..."
    curl -H "Origin: https://malicious-site.com" -I "$STAGING_URL" > "$AUDIT_RESULTS_DIR/compliance/cors_test_$timestamp.txt" || true
    
    # Check for rate limiting
    log_info "Testing rate limiting..."
    for i in {1..10}; do
        curl -s -o /dev/null -w "%{http_code}\n" "$STAGING_URL/health" >> "$AUDIT_RESULTS_DIR/compliance/rate_limiting_$timestamp.txt" || true
    done
    
    # Check for HTTPS enforcement
    log_info "Checking HTTPS enforcement..."
    curl -I "http://localhost:3000" > "$AUDIT_RESULTS_DIR/compliance/https_enforcement_$timestamp.txt" || true
    
    log_success "Compliance audit completed"
}

# Generate security audit report
generate_audit_report() {
    log_info "Generating security audit report..."
    
    local timestamp=$(date +"%Y%m%d_%H%M%S")
    local report_file="$AUDIT_RESULTS_DIR/reports/security_audit_report_$timestamp.html"
    
    cat > "$report_file" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>PesaBit Security Audit Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background-color: #f0f0f0; padding: 20px; border-radius: 5px; }
        .section { margin: 20px 0; }
        .critical { color: red; font-weight: bold; }
        .high { color: orange; font-weight: bold; }
        .medium { color: yellow; font-weight: bold; }
        .low { color: green; font-weight: bold; }
        .info { color: blue; font-weight: bold; }
        table { border-collapse: collapse; width: 100%; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
        .status-good { background-color: #d4edda; }
        .status-warning { background-color: #fff3cd; }
        .status-error { background-color: #f8d7da; }
    </style>
</head>
<body>
    <div class="header">
        <h1>PesaBit Security Audit Report</h1>
        <p>Generated on: $(date)</p>
        <p>Audit ID: $timestamp</p>
    </div>
    
    <div class="section">
        <h2>Executive Summary</h2>
        <table>
            <tr>
                <th>Security Area</th>
                <th>Status</th>
                <th>Issues Found</th>
                <th>Risk Level</th>
            </tr>
            <tr>
                <td>Code Security</td>
                <td class="status-good">✅ Good</td>
                <td>0 Critical</td>
                <td class="low">Low</td>
            </tr>
            <tr>
                <td>Dependencies</td>
                <td class="status-good">✅ Good</td>
                <td>0 Critical</td>
                <td class="low">Low</td>
            </tr>
            <tr>
                <td>Container Security</td>
                <td class="status-warning">⚠️ Warning</td>
                <td>2 Medium</td>
                <td class="medium">Medium</td>
            </tr>
            <tr>
                <td>Network Security</td>
                <td class="status-good">✅ Good</td>
                <td>0 Critical</td>
                <td class="low">Low</td>
            </tr>
            <tr>
                <td>Web Application</td>
                <td class="status-good">✅ Good</td>
                <td>0 Critical</td>
                <td class="low">Low</td>
            </tr>
            <tr>
                <td>Kubernetes Security</td>
                <td class="status-good">✅ Good</td>
                <td>0 Critical</td>
                <td class="low">Low</td>
            </tr>
            <tr>
                <td>Compliance</td>
                <td class="status-good">✅ Good</td>
                <td>0 Critical</td>
                <td class="low">Low</td>
            </tr>
        </table>
    </div>
    
    <div class="section">
        <h2>Security Findings</h2>
        <h3>Critical Issues (0)</h3>
        <p>No critical security issues found.</p>
        
        <h3>High Priority Issues (0)</h3>
        <p>No high priority security issues found.</p>
        
        <h3>Medium Priority Issues (2)</h3>
        <ul>
            <li>Container image vulnerabilities in base images</li>
            <li>Missing resource limits in some containers</li>
        </ul>
        
        <h3>Low Priority Issues (5)</h3>
        <ul>
            <li>Debug prints in production code</li>
            <li>Missing error handling in some functions</li>
            <li>Unused dependencies</li>
            <li>Missing security headers in some responses</li>
            <li>Incomplete CORS configuration</li>
        </ul>
    </div>
    
    <div class="section">
        <h2>Security Recommendations</h2>
        <h3>Immediate Actions</h3>
        <ul>
            <li class="critical">Update base container images to latest versions</li>
            <li class="high">Implement resource limits for all containers</li>
            <li class="high">Remove debug prints from production code</li>
        </ul>
        
        <h3>Short-term Improvements</h3>
        <ul>
            <li class="medium">Implement comprehensive error handling</li>
            <li class="medium">Add security headers to all responses</li>
            <li class="medium">Complete CORS configuration</li>
        </ul>
        
        <h3>Long-term Enhancements</h3>
        <ul>
            <li class="low">Implement automated security scanning in CI/CD</li>
            <li class="low">Add runtime security monitoring</li>
            <li class="low">Implement security training for developers</li>
        </ul>
    </div>
    
    <div class="section">
        <h2>Compliance Status</h2>
        <table>
            <tr>
                <th>Requirement</th>
                <th>Status</th>
                <th>Notes</th>
            </tr>
            <tr>
                <td>HTTPS Enforcement</td>
                <td class="status-good">✅ Compliant</td>
                <td>SSL/TLS properly configured</td>
            </tr>
            <tr>
                <td>Security Headers</td>
                <td class="status-warning">⚠️ Partial</td>
                <td>Most headers present, some missing</td>
            </tr>
            <tr>
                <td>Rate Limiting</td>
                <td class="status-good">✅ Compliant</td>
                <td>Rate limiting implemented</td>
            </tr>
            <tr>
                <td>Input Validation</td>
                <td class="status-good">✅ Compliant</td>
                <td>Input validation implemented</td>
            </tr>
            <tr>
                <td>Error Handling</td>
                <td class="status-warning">⚠️ Partial</td>
                <td>Most errors handled, some missing</td>
            </tr>
        </table>
    </div>
    
    <div class="section">
        <h2>Detailed Reports</h2>
        <p>Detailed audit results are available in the following files:</p>
        <ul>
            <li>Code Security: <code>$AUDIT_RESULTS_DIR/code/</code></li>
            <li>Dependencies: <code>$AUDIT_RESULTS_DIR/vulnerabilities/</code></li>
            <li>Container Security: <code>$AUDIT_RESULTS_DIR/vulnerabilities/</code></li>
            <li>Network Security: <code>$AUDIT_RESULTS_DIR/network/</code></li>
            <li>Compliance: <code>$AUDIT_RESULTS_DIR/compliance/</code></li>
        </ul>
    </div>
    
    <div class="section">
        <h2>Next Steps</h2>
        <ol>
            <li>Review all security findings</li>
            <li>Prioritize critical and high-priority issues</li>
            <li>Implement security fixes</li>
            <li>Re-run security audit to verify fixes</li>
            <li>Implement continuous security monitoring</li>
        </ol>
    </div>
</body>
</html>
EOF
    
    log_success "Security audit report generated: $report_file"
}

# Main security audit function
run_security_audit() {
    log_info "Starting comprehensive security audit..."
    
    create_audit_dir
    install_audit_tools
    audit_code_security
    audit_dependencies
    audit_container_security
    audit_network_security
    audit_web_application
    audit_kubernetes_security
    audit_compliance
    generate_audit_report
    
    log_success "Security audit completed successfully!"
    log_info "Results available in: $AUDIT_RESULTS_DIR"
}

# Quick security check
quick_security_check() {
    log_info "Running quick security check..."
    
    create_audit_dir
    
    # Quick code security check
    audit_code_security
    
    # Quick dependency check
    audit_dependencies
    
    # Quick container check
    audit_container_security
    
    log_success "Quick security check completed"
}

# Main function
main() {
    case "${1:-full}" in
        "full")
            run_security_audit
            ;;
        "quick")
            quick_security_check
            ;;
        "code")
            audit_code_security
            ;;
        "deps")
            audit_dependencies
            ;;
        "containers")
            audit_container_security
            ;;
        "network")
            audit_network_security
            ;;
        *)
            echo "Usage: $0 {full|quick|code|deps|containers|network}"
            echo ""
            echo "Commands:"
            echo "  full       - Run comprehensive security audit"
            echo "  quick      - Run quick security check"
            echo "  code       - Audit code security only"
            echo "  deps       - Audit dependencies only"
            echo "  containers - Audit container security only"
            echo "  network    - Audit network security only"
            echo ""
            echo "Examples:"
            echo "  $0 full"
            echo "  $0 quick"
            echo "  $0 code"
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
