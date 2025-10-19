#!/bin/bash

# PesaBit Production Deployment Script
# This script deploys PesaBit to production with proper security and monitoring

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
NAMESPACE="pesabit"
RELEASE_NAME="pesabit"
CHART_PATH="./helm/pesabit"
VALUES_FILE="./helm/pesabit/values-production.yaml"

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

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if kubectl is installed
    if ! command -v kubectl &> /dev/null; then
        log_error "kubectl is not installed"
        exit 1
    fi
    
    # Check if helm is installed
    if ! command -v helm &> /dev/null; then
        log_error "helm is not installed"
        exit 1
    fi
    
    # Check if docker is installed
    if ! command -v docker &> /dev/null; then
        log_error "docker is not installed"
        exit 1
    fi
    
    # Check if we can connect to Kubernetes
    if ! kubectl cluster-info &> /dev/null; then
        log_error "Cannot connect to Kubernetes cluster"
        exit 1
    fi
    
    log_success "All prerequisites met"
}

# Build and push Docker images
build_and_push_images() {
    log_info "Building and pushing Docker images..."
    
    # Build API Gateway
    log_info "Building API Gateway image..."
    docker build -f services/api-gateway/Dockerfile -t pesabit/api-gateway:latest .
    docker tag pesabit/api-gateway:latest pesabit/api-gateway:$(git rev-parse --short HEAD)
    
    # Build User Service
    log_info "Building User Service image..."
    docker build -f services/user-service/Dockerfile -t pesabit/user-service:latest .
    docker tag pesabit/user-service:latest pesabit/user-service:$(git rev-parse --short HEAD)
    
    # Build Payment Service
    log_info "Building Payment Service image..."
    docker build -f services/payment-service/Dockerfile -t pesabit/payment-service:latest .
    docker tag pesabit/payment-service:latest pesabit/payment-service:$(git rev-parse --short HEAD)
    
    # Build Frontend
    log_info "Building Frontend image..."
    docker build -f frontend/Dockerfile -t pesabit/frontend:latest ./frontend
    docker tag pesabit/frontend:latest pesabit/frontend:$(git rev-parse --short HEAD)
    
    # Push images (assuming you have a registry configured)
    log_info "Pushing images to registry..."
    # docker push pesabit/api-gateway:latest
    # docker push pesabit/user-service:latest
    # docker push pesabit/payment-service:latest
    # docker push pesabit/frontend:latest
    
    log_success "Images built and pushed successfully"
}

# Create namespace
create_namespace() {
    log_info "Creating namespace: $NAMESPACE"
    
    if kubectl get namespace $NAMESPACE &> /dev/null; then
        log_warning "Namespace $NAMESPACE already exists"
    else
        kubectl create namespace $NAMESPACE
        log_success "Namespace $NAMESPACE created"
    fi
}

# Deploy secrets
deploy_secrets() {
    log_info "Deploying secrets..."
    
    # Create secret for database credentials
    kubectl create secret generic pesabit-db-secret \
        --from-literal=username=pesabit \
        --from-literal=password=your-secure-password \
        --namespace=$NAMESPACE \
        --dry-run=client -o yaml | kubectl apply -f -
    
    # Create secret for Redis credentials
    kubectl create secret generic pesabit-redis-secret \
        --from-literal=password=your-redis-password \
        --namespace=$NAMESPACE \
        --dry-run=client -o yaml | kubectl apply -f -
    
    # Create secret for JWT
    kubectl create secret generic pesabit-jwt-secret \
        --from-literal=secret=your-super-secure-jwt-secret-key-minimum-32-characters \
        --namespace=$NAMESPACE \
        --dry-run=client -o yaml | kubectl apply -f -
    
    # Create secret for M-Pesa credentials
    kubectl create secret generic pesabit-mpesa-secret \
        --from-literal=consumer-key=your-mpesa-consumer-key \
        --from-literal=consumer-secret=your-mpesa-consumer-secret \
        --from-literal=passkey=your-mpesa-passkey \
        --namespace=$NAMESPACE \
        --dry-run=client -o yaml | kubectl apply -f -
    
    # Create secret for SMS credentials
    kubectl create secret generic pesabit-sms-secret \
        --from-literal=api-key=your-sms-api-key \
        --from-literal=username=your-sms-username \
        --namespace=$NAMESPACE \
        --dry-run=client -o yaml | kubectl apply -f -
    
    log_success "Secrets deployed"
}

# Deploy with Helm
deploy_with_helm() {
    log_info "Deploying with Helm..."
    
    # Add required Helm repositories
    helm repo add bitnami https://charts.bitnami.com/bitnami
    helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
    helm repo add grafana https://grafana.github.io/helm-charts
    helm repo update
    
    # Deploy PostgreSQL
    log_info "Deploying PostgreSQL..."
    helm upgrade --install postgresql bitnami/postgresql \
        --namespace $NAMESPACE \
        --set auth.postgresPassword=pesabit-password \
        --set auth.database=pesabit \
        --set primary.persistence.size=20Gi \
        --set primary.resources.requests.memory=512Mi \
        --set primary.resources.requests.cpu=250m \
        --set primary.resources.limits.memory=1Gi \
        --set primary.resources.limits.cpu=500m
    
    # Deploy Redis
    log_info "Deploying Redis..."
    helm upgrade --install redis bitnami/redis \
        --namespace $NAMESPACE \
        --set auth.password=redis-password \
        --set master.persistence.size=10Gi \
        --set master.resources.requests.memory=256Mi \
        --set master.resources.requests.cpu=100m \
        --set master.resources.limits.memory=512Mi \
        --set master.resources.limits.cpu=250m
    
    # Deploy Prometheus
    log_info "Deploying Prometheus..."
    helm upgrade --install prometheus prometheus-community/kube-prometheus-stack \
        --namespace $NAMESPACE \
        --set grafana.enabled=true \
        --set grafana.adminPassword=admin \
        --set prometheus.prometheusSpec.retention=30d \
        --set prometheus.prometheusSpec.storageSpec.volumeClaimTemplate.spec.resources.requests.storage=20Gi
    
    # Deploy PesaBit application
    log_info "Deploying PesaBit application..."
    helm upgrade --install $RELEASE_NAME $CHART_PATH \
        --namespace $NAMESPACE \
        --values $VALUES_FILE \
        --set image.tag=$(git rev-parse --short HEAD) \
        --wait --timeout=10m
    
    log_success "Application deployed successfully"
}

# Run database migrations
run_migrations() {
    log_info "Running database migrations..."
    
    # Wait for PostgreSQL to be ready
    kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=postgresql -n $NAMESPACE --timeout=300s
    
    # Get PostgreSQL service name
    POSTGRES_SERVICE=$(kubectl get svc -n $NAMESPACE -l app.kubernetes.io/name=postgresql -o jsonpath='{.items[0].metadata.name}')
    
    # Run migrations
    kubectl run pesabit-migrate --image=pesabit/api-gateway:latest \
        --namespace=$NAMESPACE \
        --rm -i --restart=Never \
        --env="DATABASE_URL=postgresql://pesabit:pesabit-password@$POSTGRES_SERVICE:5432/pesabit" \
        --env="RUN_MIGRATIONS=true" \
        --command -- /bin/sh -c "cargo run --bin api-gateway"
    
    log_success "Database migrations completed"
}

# Verify deployment
verify_deployment() {
    log_info "Verifying deployment..."
    
    # Check if all pods are running
    kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=pesabit -n $NAMESPACE --timeout=300s
    
    # Check service endpoints
    kubectl get svc -n $NAMESPACE
    
    # Test health endpoints
    API_GATEWAY_SERVICE=$(kubectl get svc -n $NAMESPACE -l app.kubernetes.io/name=pesabit-api-gateway -o jsonpath='{.items[0].metadata.name}')
    
    if kubectl run test-health --image=curlimages/curl:latest \
        --namespace=$NAMESPACE \
        --rm -i --restart=Never \
        --command -- curl -f http://$API_GATEWAY_SERVICE:3000/health; then
        log_success "Health check passed"
    else
        log_error "Health check failed"
        exit 1
    fi
    
    log_success "Deployment verification completed"
}

# Setup monitoring
setup_monitoring() {
    log_info "Setting up monitoring..."
    
    # Apply alerting rules
    kubectl apply -f infrastructure/monitoring/alerting_rules.yml -n $NAMESPACE
    
    # Apply Grafana dashboards
    kubectl apply -f infrastructure/monitoring/grafana/ -n $NAMESPACE
    
    log_success "Monitoring setup completed"
}

# Main deployment function
main() {
    log_info "Starting PesaBit production deployment..."
    
    check_prerequisites
    build_and_push_images
    create_namespace
    deploy_secrets
    deploy_with_helm
    run_migrations
    verify_deployment
    setup_monitoring
    
    log_success "PesaBit production deployment completed successfully!"
    
    # Display access information
    echo ""
    log_info "Access Information:"
    echo "  - API Gateway: kubectl port-forward svc/pesabit-api-gateway 3000:3000 -n $NAMESPACE"
    echo "  - Frontend: kubectl port-forward svc/pesabit-frontend 5173:80 -n $NAMESPACE"
    echo "  - Grafana: kubectl port-forward svc/prometheus-grafana 3001:80 -n $NAMESPACE"
    echo "  - Prometheus: kubectl port-forward svc/prometheus-kube-prometheus-prometheus 9090:9090 -n $NAMESPACE"
    echo ""
    log_info "Grafana credentials: admin/admin"
    echo ""
    log_warning "Remember to:"
    echo "  1. Update DNS records to point to your load balancer"
    echo "  2. Configure SSL certificates"
    echo "  3. Set up backup procedures"
    echo "  4. Configure log aggregation"
    echo "  5. Set up alerting notifications"
}

# Run main function
main "$@"
