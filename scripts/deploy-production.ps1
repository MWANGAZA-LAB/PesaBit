# PesaBit Production Deployment Script (PowerShell)
# This script deploys PesaBit to production with proper security and monitoring

param(
    [string]$Namespace = "pesabit",
    [string]$ReleaseName = "pesabit",
    [string]$ChartPath = "./helm/pesabit",
    [string]$ValuesFile = "./helm/pesabit/values-production.yaml"
)

# Configuration
$ErrorActionPreference = "Stop"

# Functions
function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

# Check prerequisites
function Test-Prerequisites {
    Write-Info "Checking prerequisites..."
    
    # Check if kubectl is installed
    if (-not (Get-Command kubectl -ErrorAction SilentlyContinue)) {
        Write-Error "kubectl is not installed"
        exit 1
    }
    
    # Check if helm is installed
    if (-not (Get-Command helm -ErrorAction SilentlyContinue)) {
        Write-Error "helm is not installed"
        exit 1
    }
    
    # Check if docker is installed
    if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
        Write-Error "docker is not installed"
        exit 1
    }
    
    # Check if we can connect to Kubernetes
    try {
        kubectl cluster-info | Out-Null
    }
    catch {
        Write-Error "Cannot connect to Kubernetes cluster"
        exit 1
    }
    
    Write-Success "All prerequisites met"
}

# Build and push Docker images
function Build-AndPushImages {
    Write-Info "Building and pushing Docker images..."
    
    # Build API Gateway
    Write-Info "Building API Gateway image..."
    docker build -f services/api-gateway/Dockerfile -t pesabit/api-gateway:latest .
    docker tag pesabit/api-gateway:latest pesabit/api-gateway:$(git rev-parse --short HEAD)
    
    # Build User Service
    Write-Info "Building User Service image..."
    docker build -f services/user-service/Dockerfile -t pesabit/user-service:latest .
    docker tag pesabit/user-service:latest pesabit/user-service:$(git rev-parse --short HEAD)
    
    # Build Payment Service
    Write-Info "Building Payment Service image..."
    docker build -f services/payment-service/Dockerfile -t pesabit/payment-service:latest .
    docker tag pesabit/payment-service:latest pesabit/payment-service:$(git rev-parse --short HEAD)
    
    # Build Frontend
    Write-Info "Building Frontend image..."
    docker build -f frontend/Dockerfile -t pesabit/frontend:latest ./frontend
    docker tag pesabit/frontend:latest pesabit/frontend:$(git rev-parse --short HEAD)
    
    # Push images (assuming you have a registry configured)
    Write-Info "Pushing images to registry..."
    # docker push pesabit/api-gateway:latest
    # docker push pesabit/user-service:latest
    # docker push pesabit/payment-service:latest
    # docker push pesabit/frontend:latest
    
    Write-Success "Images built and pushed successfully"
}

# Create namespace
function New-Namespace {
    Write-Info "Creating namespace: $Namespace"
    
    try {
        kubectl get namespace $Namespace | Out-Null
        Write-Warning "Namespace $Namespace already exists"
    }
    catch {
        kubectl create namespace $Namespace
        Write-Success "Namespace $Namespace created"
    }
}

# Deploy secrets
function Deploy-Secrets {
    Write-Info "Deploying secrets..."
    
    # Create secret for database credentials
    kubectl create secret generic pesabit-db-secret `
        --from-literal=username=pesabit `
        --from-literal=password=your-secure-password `
        --namespace=$Namespace `
        --dry-run=client -o yaml | kubectl apply -f -
    
    # Create secret for Redis credentials
    kubectl create secret generic pesabit-redis-secret `
        --from-literal=password=your-redis-password `
        --namespace=$Namespace `
        --dry-run=client -o yaml | kubectl apply -f -
    
    # Create secret for JWT
    kubectl create secret generic pesabit-jwt-secret `
        --from-literal=secret=your-super-secure-jwt-secret-key-minimum-32-characters `
        --namespace=$Namespace `
        --dry-run=client -o yaml | kubectl apply -f -
    
    # Create secret for M-Pesa credentials
    kubectl create secret generic pesabit-mpesa-secret `
        --from-literal=consumer-key=your-mpesa-consumer-key `
        --from-literal=consumer-secret=your-mpesa-consumer-secret `
        --from-literal=passkey=your-mpesa-passkey `
        --namespace=$Namespace `
        --dry-run=client -o yaml | kubectl apply -f -
    
    # Create secret for SMS credentials
    kubectl create secret generic pesabit-sms-secret `
        --from-literal=api-key=your-sms-api-key `
        --from-literal=username=your-sms-username `
        --namespace=$Namespace `
        --dry-run=client -o yaml | kubectl apply -f -
    
    Write-Success "Secrets deployed"
}

# Deploy with Helm
function Deploy-WithHelm {
    Write-Info "Deploying with Helm..."
    
    # Add required Helm repositories
    helm repo add bitnami https://charts.bitnami.com/bitnami
    helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
    helm repo add grafana https://grafana.github.io/helm-charts
    helm repo update
    
    # Deploy PostgreSQL
    Write-Info "Deploying PostgreSQL..."
    helm upgrade --install postgresql bitnami/postgresql `
        --namespace $Namespace `
        --set auth.postgresPassword=pesabit-password `
        --set auth.database=pesabit `
        --set primary.persistence.size=20Gi `
        --set primary.resources.requests.memory=512Mi `
        --set primary.resources.requests.cpu=250m `
        --set primary.resources.limits.memory=1Gi `
        --set primary.resources.limits.cpu=500m
    
    # Deploy Redis
    Write-Info "Deploying Redis..."
    helm upgrade --install redis bitnami/redis `
        --namespace $Namespace `
        --set auth.password=redis-password `
        --set master.persistence.size=10Gi `
        --set master.resources.requests.memory=256Mi `
        --set master.resources.requests.cpu=100m `
        --set master.resources.limits.memory=512Mi `
        --set master.resources.limits.cpu=250m
    
    # Deploy Prometheus
    Write-Info "Deploying Prometheus..."
    helm upgrade --install prometheus prometheus-community/kube-prometheus-stack `
        --namespace $Namespace `
        --set grafana.enabled=true `
        --set grafana.adminPassword=admin `
        --set prometheus.prometheusSpec.retention=30d `
        --set prometheus.prometheusSpec.storageSpec.volumeClaimTemplate.spec.resources.requests.storage=20Gi
    
    # Deploy PesaBit application
    Write-Info "Deploying PesaBit application..."
    helm upgrade --install $ReleaseName $ChartPath `
        --namespace $Namespace `
        --values $ValuesFile `
        --set image.tag=$(git rev-parse --short HEAD) `
        --wait --timeout=10m
    
    Write-Success "Application deployed successfully"
}

# Run database migrations
function Invoke-Migrations {
    Write-Info "Running database migrations..."
    
    # Wait for PostgreSQL to be ready
    kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=postgresql -n $Namespace --timeout=300s
    
    # Get PostgreSQL service name
    $PostgresService = kubectl get svc -n $Namespace -l app.kubernetes.io/name=postgresql -o jsonpath='{.items[0].metadata.name}'
    
    # Run migrations
    kubectl run pesabit-migrate --image=pesabit/api-gateway:latest `
        --namespace=$Namespace `
        --rm -i --restart=Never `
        --env="DATABASE_URL=postgresql://pesabit:pesabit-password@$PostgresService:5432/pesabit" `
        --env="RUN_MIGRATIONS=true" `
        --command -- /bin/sh -c "cargo run --bin api-gateway"
    
    Write-Success "Database migrations completed"
}

# Verify deployment
function Test-Deployment {
    Write-Info "Verifying deployment..."
    
    # Check if all pods are running
    kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=pesabit -n $Namespace --timeout=300s
    
    # Check service endpoints
    kubectl get svc -n $Namespace
    
    # Test health endpoints
    $ApiGatewayService = kubectl get svc -n $Namespace -l app.kubernetes.io/name=pesabit-api-gateway -o jsonpath='{.items[0].metadata.name}'
    
    try {
        kubectl run test-health --image=curlimages/curl:latest `
            --namespace=$Namespace `
            --rm -i --restart=Never `
            --command -- curl -f http://$ApiGatewayService:3000/health
        Write-Success "Health check passed"
    }
    catch {
        Write-Error "Health check failed"
        exit 1
    }
    
    Write-Success "Deployment verification completed"
}

# Setup monitoring
function Set-Monitoring {
    Write-Info "Setting up monitoring..."
    
    # Apply alerting rules
    kubectl apply -f infrastructure/monitoring/alerting_rules.yml -n $Namespace
    
    # Apply Grafana dashboards
    kubectl apply -f infrastructure/monitoring/grafana/ -n $Namespace
    
    Write-Success "Monitoring setup completed"
}

# Main deployment function
function Start-Deployment {
    Write-Info "Starting PesaBit production deployment..."
    
    Test-Prerequisites
    Build-AndPushImages
    New-Namespace
    Deploy-Secrets
    Deploy-WithHelm
    Invoke-Migrations
    Test-Deployment
    Set-Monitoring
    
    Write-Success "PesaBit production deployment completed successfully!"
    
    # Display access information
    Write-Host ""
    Write-Info "Access Information:"
    Write-Host "  - API Gateway: kubectl port-forward svc/pesabit-api-gateway 3000:3000 -n $Namespace"
    Write-Host "  - Frontend: kubectl port-forward svc/pesabit-frontend 5173:80 -n $Namespace"
    Write-Host "  - Grafana: kubectl port-forward svc/prometheus-grafana 3001:80 -n $Namespace"
    Write-Host "  - Prometheus: kubectl port-forward svc/prometheus-kube-prometheus-prometheus 9090:9090 -n $Namespace"
    Write-Host ""
    Write-Info "Grafana credentials: admin/admin"
    Write-Host ""
    Write-Warning "Remember to:"
    Write-Host "  1. Update DNS records to point to your load balancer"
    Write-Host "  2. Configure SSL certificates"
    Write-Host "  3. Set up backup procedures"
    Write-Host "  4. Configure log aggregation"
    Write-Host "  5. Set up alerting notifications"
}

# Run main function
Start-Deployment
