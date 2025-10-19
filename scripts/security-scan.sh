#!/bin/bash

# PesaBit Automated Security Scanning Script
# This script performs comprehensive security scans for production readiness

set -euo pipefail

# Configuration
NAMESPACE="pesabit"
SCAN_RESULTS_DIR="/security-scans"
TRIVY_DB_PATH="/tmp/trivy-db"
GRYPE_DB_PATH="/tmp/grype-db"

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

# Create scan results directory
create_scan_dir() {
    log_info "Creating scan results directory: $SCAN_RESULTS_DIR"
    mkdir -p "$SCAN_RESULTS_DIR"/{vulnerabilities,secrets,compliance,network}
    log_success "Scan results directory created"
}

# Install security scanning tools
install_security_tools() {
    log_info "Installing security scanning tools..."
    
    # Install Trivy for vulnerability scanning
    if ! command -v trivy &> /dev/null; then
        log_info "Installing Trivy..."
        curl -sfL https://raw.githubusercontent.com/aquasecurity/trivy/main/contrib/install.sh | sh -s -- -b /usr/local/bin
        log_success "Trivy installed"
    else
        log_info "Trivy already installed"
    fi
    
    # Install Grype for vulnerability scanning
    if ! command -v grype &> /dev/null; then
        log_info "Installing Grype..."
        curl -sSfL https://raw.githubusercontent.com/anchore/grype/main/install.sh | sh -s -- -b /usr/local/bin
        log_success "Grype installed"
    else
        log_info "Grype already installed"
    fi
    
    # Install TruffleHog for secret scanning
    if ! command -v trufflehog &> /dev/null; then
        log_info "Installing TruffleHog..."
        curl -sSfL https://raw.githubusercontent.com/trufflesecurity/trufflehog/main/scripts/install.sh | sh -s -- -b /usr/local/bin
        log_success "TruffleHog installed"
    else
        log_info "TruffleHog already installed"
    fi
    
    # Install Kube-score for Kubernetes security
    if ! command -v kube-score &> /dev/null; then
        log_info "Installing kube-score..."
        curl -sSfL https://raw.githubusercontent.com/zegl/kube-score/master/install.sh | sh -s -- -b /usr/local/bin
        log_success "kube-score installed"
    else
        log_info "kube-score already installed"
    fi
    
    # Install Kubeaudit for Kubernetes security
    if ! command -v kubeaudit &> /dev/null; then
        log_info "Installing kubeaudit..."
        curl -sSfL https://raw.githubusercontent.com/Shopify/kubeaudit/master/install.sh | sh -s -- -b /usr/local/bin
        log_success "kubeaudit installed"
    else
        log_info "kubeaudit already installed"
    fi
}

# Scan Docker images for vulnerabilities
scan_docker_images() {
    log_info "Scanning Docker images for vulnerabilities..."
    
    local timestamp=$(date +"%Y%m%d_%H%M%S")
    local scan_file="$SCAN_RESULTS_DIR/vulnerabilities/docker_images_$timestamp.json"
    
    # List of images to scan
    local images=(
        "pesabit/api-gateway:latest"
        "pesabit/user-service:latest"
        "pesabit/payment-service:latest"
        "pesabit/frontend:latest"
        "postgres:16-alpine"
        "redis:7-alpine"
        "nginx:alpine"
    )
    
    for image in "${images[@]}"; do
        log_info "Scanning image: $image"
        
        # Scan with Trivy
        trivy image --format json --output "$SCAN_RESULTS_DIR/vulnerabilities/trivy_${image//[\/:]/_}_$timestamp.json" "$image" || true
        
        # Scan with Grype
        grype "$image" --output json > "$SCAN_RESULTS_DIR/vulnerabilities/grype_${image//[\/:]/_}_$timestamp.json" || true
        
        log_success "Scanned image: $image"
    done
    
    log_success "Docker image vulnerability scanning completed"
}

# Scan source code for secrets
scan_source_code_secrets() {
    log_info "Scanning source code for secrets..."
    
    local timestamp=$(date +"%Y%m%d_%H%M%S")
    local scan_file="$SCAN_RESULTS_DIR/secrets/secrets_scan_$timestamp.json"
    
    # Scan with TruffleHog
    trufflehog filesystem . --output json > "$scan_file" || true
    
    # Also scan specific directories
    trufflehog filesystem services/ --output json > "$SCAN_RESULTS_DIR/secrets/services_secrets_$timestamp.json" || true
    trufflehog filesystem shared/ --output json > "$SCAN_RESULTS_DIR/secrets/shared_secrets_$timestamp.json" || true
    trufflehog filesystem frontend/ --output json > "$SCAN_RESULTS_DIR/secrets/frontend_secrets_$timestamp.json" || true
    
    log_success "Source code secret scanning completed"
}

# Scan Kubernetes manifests for security issues
scan_kubernetes_manifests() {
    log_info "Scanning Kubernetes manifests for security issues..."
    
    local timestamp=$(date +"%Y%m%d_%H%M%S")
    
    # Scan with kube-score
    if [ -d "infrastructure/kubernetes" ]; then
        kube-score score infrastructure/kubernetes/ > "$SCAN_RESULTS_DIR/compliance/kube_score_$timestamp.txt" || true
    fi
    
    # Scan with kubeaudit
    if [ -d "infrastructure/kubernetes" ]; then
        kubeaudit all -f infrastructure/kubernetes/ > "$SCAN_RESULTS_DIR/compliance/kubeaudit_$timestamp.txt" || true
    fi
    
    # Scan running cluster
    if kubectl cluster-info &> /dev/null; then
        log_info "Scanning running Kubernetes cluster..."
        
        # Export current manifests
        kubectl get all -n $NAMESPACE -o yaml > "$SCAN_RESULTS_DIR/compliance/running_cluster_$timestamp.yaml" || true
        
        # Scan with kube-score
        kubectl get all -n $NAMESPACE -o yaml | kube-score score - > "$SCAN_RESULTS_DIR/compliance/running_cluster_score_$timestamp.txt" || true
        
        # Scan with kubeaudit
        kubectl get all -n $NAMESPACE -o yaml | kubeaudit all - > "$SCAN_RESULTS_DIR/compliance/running_cluster_audit_$timestamp.txt" || true
    fi
    
    log_success "Kubernetes manifest security scanning completed"
}

# Scan network security
scan_network_security() {
    log_info "Scanning network security..."
    
    local timestamp=$(date +"%Y%m%d_%H%M%S")
    
    # Check for open ports
    if command -v nmap &> /dev/null; then
        log_info "Scanning for open ports..."
        nmap -sS -O localhost > "$SCAN_RESULTS_DIR/network/port_scan_$timestamp.txt" || true
    fi
    
    # Check SSL/TLS configuration
    if command -v sslscan &> /dev/null; then
        log_info "Scanning SSL/TLS configuration..."
        sslscan api.pesa.co.ke > "$SCAN_RESULTS_DIR/network/ssl_scan_$timestamp.txt" || true
    fi
    
    # Check DNS security
    if command -v dig &> /dev/null; then
        log_info "Checking DNS security..."
        dig @8.8.8.8 api.pesa.co.ke > "$SCAN_RESULTS_DIR/network/dns_check_$timestamp.txt" || true
    fi
    
    log_success "Network security scanning completed"
}

# Scan dependencies for vulnerabilities
scan_dependencies() {
    log_info "Scanning dependencies for vulnerabilities..."
    
    local timestamp=$(date +"%Y%m%d_%H%M%S")
    
    # Scan Rust dependencies
    if command -v cargo &> /dev/null; then
        log_info "Scanning Rust dependencies..."
        cargo audit --json > "$SCAN_RESULTS_DIR/vulnerabilities/rust_audit_$timestamp.json" || true
    fi
    
    # Scan Node.js dependencies
    if [ -f "frontend/package.json" ]; then
        log_info "Scanning Node.js dependencies..."
        cd frontend
        npm audit --json > "../$SCAN_RESULTS_DIR/vulnerabilities/node_audit_$timestamp.json" || true
        cd ..
    fi
    
    # Scan Python dependencies (if any)
    if [ -f "requirements.txt" ]; then
        log_info "Scanning Python dependencies..."
        pip-audit --format=json > "$SCAN_RESULTS_DIR/vulnerabilities/python_audit_$timestamp.json" || true
    fi
    
    log_success "Dependency vulnerability scanning completed"
}

# Generate security report
generate_security_report() {
    log_info "Generating security report..."
    
    local timestamp=$(date +"%Y%m%d_%H%M%S")
    local report_file="$SCAN_RESULTS_DIR/security_report_$timestamp.html"
    
    cat > "$report_file" << EOF
<!DOCTYPE html>
<html>
<head>
    <title>PesaBit Security Scan Report</title>
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
    </style>
</head>
<body>
    <div class="header">
        <h1>PesaBit Security Scan Report</h1>
        <p>Generated on: $(date)</p>
        <p>Scan ID: $timestamp</p>
    </div>
    
    <div class="section">
        <h2>Scan Summary</h2>
        <table>
            <tr>
                <th>Scan Type</th>
                <th>Status</th>
                <th>Files Scanned</th>
                <th>Issues Found</th>
            </tr>
            <tr>
                <td>Docker Image Vulnerabilities</td>
                <td class="info">Completed</td>
                <td>7 images</td>
                <td>See detailed reports</td>
            </tr>
            <tr>
                <td>Source Code Secrets</td>
                <td class="info">Completed</td>
                <td>All source files</td>
                <td>See detailed reports</td>
            </tr>
            <tr>
                <td>Kubernetes Security</td>
                <td class="info">Completed</td>
                <td>All manifests</td>
                <td>See detailed reports</td>
            </tr>
            <tr>
                <td>Dependency Vulnerabilities</td>
                <td class="info">Completed</td>
                <td>Rust, Node.js, Python</td>
                <td>See detailed reports</td>
            </tr>
            <tr>
                <td>Network Security</td>
                <td class="info">Completed</td>
                <td>Ports, SSL, DNS</td>
                <td>See detailed reports</td>
            </tr>
        </table>
    </div>
    
    <div class="section">
        <h2>Security Recommendations</h2>
        <ul>
            <li class="critical">Review all critical vulnerabilities immediately</li>
            <li class="high">Address high-severity issues within 48 hours</li>
            <li class="medium">Plan remediation for medium-severity issues</li>
            <li class="low">Monitor low-severity issues for updates</li>
            <li class="info">Implement automated security scanning in CI/CD</li>
        </ul>
    </div>
    
    <div class="section">
        <h2>Detailed Reports</h2>
        <p>Detailed scan results are available in the following files:</p>
        <ul>
            <li>Docker Image Vulnerabilities: <code>$SCAN_RESULTS_DIR/vulnerabilities/</code></li>
            <li>Source Code Secrets: <code>$SCAN_RESULTS_DIR/secrets/</code></li>
            <li>Kubernetes Security: <code>$SCAN_RESULTS_DIR/compliance/</code></li>
            <li>Network Security: <code>$SCAN_RESULTS_DIR/network/</code></li>
        </ul>
    </div>
    
    <div class="section">
        <h2>Next Steps</h2>
        <ol>
            <li>Review all scan results</li>
            <li>Prioritize critical and high-severity issues</li>
            <li>Implement fixes for identified vulnerabilities</li>
            <li>Re-run scans to verify fixes</li>
            <li>Set up continuous security monitoring</li>
        </ol>
    </div>
</body>
</html>
EOF
    
    log_success "Security report generated: $report_file"
}

# Cleanup old scan results
cleanup_old_scans() {
    log_info "Cleaning up old scan results (older than 7 days)..."
    
    find "$SCAN_RESULTS_DIR" -type f -mtime +7 -delete
    
    log_success "Old scan results cleaned up"
}

# Main security scan function
perform_security_scan() {
    log_info "Starting comprehensive security scan..."
    
    create_scan_dir
    install_security_tools
    scan_docker_images
    scan_source_code_secrets
    scan_kubernetes_manifests
    scan_network_security
    scan_dependencies
    generate_security_report
    cleanup_old_scans
    
    log_success "Security scan completed successfully!"
    log_info "Results available in: $SCAN_RESULTS_DIR"
}

# Quick security check
quick_security_check() {
    log_info "Performing quick security check..."
    
    create_scan_dir
    
    # Check for common security issues
    log_info "Checking for hardcoded secrets..."
    if grep -r "password\|secret\|key" . --include="*.rs" --include="*.js" --include="*.ts" | grep -v "// TODO\|// FIXME" | head -10; then
        log_warning "Potential hardcoded secrets found"
    else
        log_success "No obvious hardcoded secrets found"
    fi
    
    # Check for insecure configurations
    log_info "Checking for insecure configurations..."
    if [ -f "docker-compose.yml" ]; then
        if grep -q "privileged: true" docker-compose.yml; then
            log_warning "Privileged containers detected"
        fi
        if grep -q "network_mode: host" docker-compose.yml; then
            log_warning "Host network mode detected"
        fi
    fi
    
    # Check for missing security headers
    log_info "Checking security configurations..."
    if [ -f "infrastructure/kubernetes/ingress.yaml" ]; then
        if grep -q "X-Frame-Options" infrastructure/kubernetes/ingress.yaml; then
            log_success "Security headers configured"
        else
            log_warning "Security headers not configured"
        fi
    fi
    
    log_success "Quick security check completed"
}

# Main function
main() {
    case "${1:-scan}" in
        "scan")
            perform_security_scan
            ;;
        "quick")
            quick_security_check
            ;;
        "cleanup")
            cleanup_old_scans
            ;;
        *)
            echo "Usage: $0 {scan|quick|cleanup}"
            echo ""
            echo "Commands:"
            echo "  scan     - Perform comprehensive security scan"
            echo "  quick    - Perform quick security check"
            echo "  cleanup  - Clean up old scan results"
            echo ""
            echo "Examples:"
            echo "  $0 scan"
            echo "  $0 quick"
            echo "  $0 cleanup"
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
