#!/bin/bash

# PesaBit Production Deployment Script
# This script deploys PesaBit to production Kubernetes cluster

set -euo pipefail

# Configuration
NAMESPACE="pesabit"
CLUSTER_NAME="pesabit-production"
REGION="us-east-1"
IMAGE_TAG="latest"
REPLICAS=3

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
    
    # Check if aws cli is installed
    if ! command -v aws &> /dev/null; then
        log_error "aws cli is not installed"
        exit 1
    fi
    
    log_success "All prerequisites met"
}

# Configure AWS EKS
configure_aws_eks() {
    log_info "Configuring AWS EKS..."
    
    # Update kubeconfig
    aws eks update-kubeconfig --region $REGION --name $CLUSTER_NAME
    
    # Verify cluster access
    if kubectl cluster-info &> /dev/null; then
        log_success "EKS cluster access configured"
    else
        log_error "Failed to access EKS cluster"
        exit 1
    fi
}

# Create namespace
create_namespace() {
    log_info "Creating namespace: $NAMESPACE"
    
    kubectl create namespace $NAMESPACE --dry-run=client -o yaml | kubectl apply -f -
    
    # Add labels
    kubectl label namespace $NAMESPACE name=$NAMESPACE --overwrite
    kubectl label namespace $NAMESPACE environment=production --overwrite
    
    log_success "Namespace created: $NAMESPACE"
}

# Create secrets
create_secrets() {
    log_info "Creating production secrets..."
    
    # Database secret
    kubectl create secret generic pesabit-db-secret \
        --from-literal=username=pesabit_prod \
        --from-literal=password="${DB_PASSWORD}" \
        --from-literal=database=pesabit_prod \
        --namespace=$NAMESPACE \
        --dry-run=client -o yaml | kubectl apply -f -
    
    # Redis secret
    kubectl create secret generic pesabit-redis-secret \
        --from-literal=password="${REDIS_PASSWORD}" \
        --namespace=$NAMESPACE \
        --dry-run=client -o yaml | kubectl apply -f -
    
    # JWT secret
    kubectl create secret generic pesabit-jwt-secret \
        --from-literal=secret="${JWT_SECRET}" \
        --namespace=$NAMESPACE \
        --dry-run=client -o yaml | kubectl apply -f -
    
    # M-Pesa secrets
    kubectl create secret generic pesabit-mpesa-secret \
        --from-literal=consumer-key="${MPESA_CONSUMER_KEY}" \
        --from-literal=consumer-secret="${MPESA_CONSUMER_SECRET}" \
        --from-literal=passkey="${MPESA_PASSKEY}" \
        --namespace=$NAMESPACE \
        --dry-run=client -o yaml | kubectl apply -f -
    
    # Lightning Network secrets
    kubectl create secret generic pesabit-lightning-secret \
        --from-literal=macaroon="${LIGHTNING_MACAROON}" \
        --from-literal=tls-cert="${LIGHTNING_TLS_CERT}" \
        --namespace=$NAMESPACE \
        --dry-run=client -o yaml | kubectl apply -f -
    
    log_success "Production secrets created"
}

# Deploy PostgreSQL
deploy_postgresql() {
    log_info "Deploying PostgreSQL..."
    
    # Create PostgreSQL deployment
    cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: Deployment
metadata:
  name: postgresql
  namespace: $NAMESPACE
spec:
  replicas: 1
  selector:
    matchLabels:
      app: postgresql
  template:
    metadata:
      labels:
        app: postgresql
    spec:
      containers:
      - name: postgresql
        image: postgres:16-alpine
        env:
        - name: POSTGRES_DB
          valueFrom:
            secretKeyRef:
              name: pesabit-db-secret
              key: database
        - name: POSTGRES_USER
          valueFrom:
            secretKeyRef:
              name: pesabit-db-secret
              key: username
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: pesabit-db-secret
              key: password
        ports:
        - containerPort: 5432
        volumeMounts:
        - name: postgres-storage
          mountPath: /var/lib/postgresql/data
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
      volumes:
      - name: postgres-storage
        persistentVolumeClaim:
          claimName: postgres-pvc
---
apiVersion: v1
kind: Service
metadata:
  name: postgresql
  namespace: $NAMESPACE
spec:
  selector:
    app: postgresql
  ports:
  - port: 5432
    targetPort: 5432
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: postgres-pvc
  namespace: $NAMESPACE
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 20Gi
EOF
    
    # Wait for PostgreSQL to be ready
    kubectl wait --for=condition=ready pod -l app=postgresql -n $NAMESPACE --timeout=300s
    
    log_success "PostgreSQL deployed"
}

# Deploy Redis
deploy_redis() {
    log_info "Deploying Redis..."
    
    # Create Redis deployment
    cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: Deployment
metadata:
  name: redis
  namespace: $NAMESPACE
spec:
  replicas: 1
  selector:
    matchLabels:
      app: redis
  template:
    metadata:
      labels:
        app: redis
    spec:
      containers:
      - name: redis
        image: redis:7-alpine
        command: ["redis-server", "--requirepass", "\$(REDIS_PASSWORD)"]
        env:
        - name: REDIS_PASSWORD
          valueFrom:
            secretKeyRef:
              name: pesabit-redis-secret
              key: password
        ports:
        - containerPort: 6379
        resources:
          requests:
            memory: "256Mi"
            cpu: "100m"
          limits:
            memory: "1Gi"
            cpu: "500m"
---
apiVersion: v1
kind: Service
metadata:
  name: redis
  namespace: $NAMESPACE
spec:
  selector:
    app: redis
  ports:
  - port: 6379
    targetPort: 6379
EOF
    
    # Wait for Redis to be ready
    kubectl wait --for=condition=ready pod -l app=redis -n $NAMESPACE --timeout=300s
    
    log_success "Redis deployed"
}

# Deploy API Gateway
deploy_api_gateway() {
    log_info "Deploying API Gateway..."
    
    # Create API Gateway deployment
    cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-gateway
  namespace: $NAMESPACE
spec:
  replicas: $REPLICAS
  selector:
    matchLabels:
      app: api-gateway
  template:
    metadata:
      labels:
        app: api-gateway
    spec:
      containers:
      - name: api-gateway
        image: pesabit/api-gateway:$IMAGE_TAG
        env:
        - name: APP_ENV
          value: "production"
        - name: SERVICE_PORT
          value: "3000"
        - name: DATABASE_URL
          value: "postgresql://pesabit_prod:\$(DB_PASSWORD)@postgresql:5432/pesabit_prod"
        - name: DB_PASSWORD
          valueFrom:
            secretKeyRef:
              name: pesabit-db-secret
              key: password
        - name: REDIS_URL
          value: "redis://:\$(REDIS_PASSWORD)@redis:6379"
        - name: REDIS_PASSWORD
          valueFrom:
            secretKeyRef:
              name: pesabit-redis-secret
              key: password
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: pesabit-jwt-secret
              key: secret
        - name: USER_SERVICE_URL
          value: "http://user-service:8001"
        - name: PAYMENT_SERVICE_URL
          value: "http://payment-service:8002"
        - name: RATE_LIMIT_REQUESTS_PER_MINUTE
          value: "1000"
        - name: CORS_ORIGINS
          value: "https://app.pesa.co.ke,https://pesa.co.ke"
        - name: RUST_LOG
          value: "info,api_gateway=info"
        ports:
        - containerPort: 3000
        resources:
          requests:
            memory: "256Mi"
            cpu: "100m"
          limits:
            memory: "1Gi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: api-gateway
  namespace: $NAMESPACE
spec:
  selector:
    app: api-gateway
  ports:
  - port: 3000
    targetPort: 3000
  type: ClusterIP
EOF
    
    # Wait for API Gateway to be ready
    kubectl wait --for=condition=ready pod -l app=api-gateway -n $NAMESPACE --timeout=300s
    
    log_success "API Gateway deployed"
}

# Deploy User Service
deploy_user_service() {
    log_info "Deploying User Service..."
    
    # Create User Service deployment
    cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: Deployment
metadata:
  name: user-service
  namespace: $NAMESPACE
spec:
  replicas: $REPLICAS
  selector:
    matchLabels:
      app: user-service
  template:
    metadata:
      labels:
        app: user-service
    spec:
      containers:
      - name: user-service
        image: pesabit/user-service:$IMAGE_TAG
        env:
        - name: APP_ENV
          value: "production"
        - name: SERVICE_PORT
          value: "8001"
        - name: DATABASE_URL
          value: "postgresql://pesabit_prod:\$(DB_PASSWORD)@postgresql:5432/pesabit_prod"
        - name: DB_PASSWORD
          valueFrom:
            secretKeyRef:
              name: pesabit-db-secret
              key: password
        - name: REDIS_URL
          value: "redis://:\$(REDIS_PASSWORD)@redis:6379"
        - name: REDIS_PASSWORD
          valueFrom:
            secretKeyRef:
              name: pesabit-redis-secret
              key: password
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: pesabit-jwt-secret
              key: secret
        - name: RUST_LOG
          value: "info,user_service=info"
        ports:
        - containerPort: 8001
        resources:
          requests:
            memory: "256Mi"
            cpu: "100m"
          limits:
            memory: "1Gi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8001
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8001
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: user-service
  namespace: $NAMESPACE
spec:
  selector:
    app: user-service
  ports:
  - port: 8001
    targetPort: 8001
  type: ClusterIP
EOF
    
    # Wait for User Service to be ready
    kubectl wait --for=condition=ready pod -l app=user-service -n $NAMESPACE --timeout=300s
    
    log_success "User Service deployed"
}

# Deploy Payment Service
deploy_payment_service() {
    log_info "Deploying Payment Service..."
    
    # Create Payment Service deployment
    cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: Deployment
metadata:
  name: payment-service
  namespace: $NAMESPACE
spec:
  replicas: $REPLICAS
  selector:
    matchLabels:
      app: payment-service
  template:
    metadata:
      labels:
        app: payment-service
    spec:
      containers:
      - name: payment-service
        image: pesabit/payment-service:$IMAGE_TAG
        env:
        - name: APP_ENV
          value: "production"
        - name: SERVICE_PORT
          value: "8002"
        - name: DATABASE_URL
          value: "postgresql://pesabit_prod:\$(DB_PASSWORD)@postgresql:5432/pesabit_prod"
        - name: DB_PASSWORD
          valueFrom:
            secretKeyRef:
              name: pesabit-db-secret
              key: password
        - name: REDIS_URL
          value: "redis://:\$(REDIS_PASSWORD)@redis:6379"
        - name: REDIS_PASSWORD
          valueFrom:
            secretKeyRef:
              name: pesabit-redis-secret
              key: password
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: pesabit-jwt-secret
              key: secret
        - name: MPESA_CONSUMER_KEY
          valueFrom:
            secretKeyRef:
              name: pesabit-mpesa-secret
              key: consumer-key
        - name: MPESA_CONSUMER_SECRET
          valueFrom:
            secretKeyRef:
              name: pesabit-mpesa-secret
              key: consumer-secret
        - name: MPESA_PASSKEY
          valueFrom:
            secretKeyRef:
              name: pesabit-mpesa-secret
              key: passkey
        - name: MPESA_PRODUCTION_URL
          value: "https://api.safaricom.co.ke"
        - name: LIGHTNING_MACAROON
          valueFrom:
            secretKeyRef:
              name: pesabit-lightning-secret
              key: macaroon
        - name: LIGHTNING_TLS_CERT
          valueFrom:
            secretKeyRef:
              name: pesabit-lightning-secret
              key: tls-cert
        - name: RUST_LOG
          value: "info,payment_service=info"
        ports:
        - containerPort: 8002
        resources:
          requests:
            memory: "512Mi"
            cpu: "200m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8002
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8002
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: payment-service
  namespace: $NAMESPACE
spec:
  selector:
    app: payment-service
  ports:
  - port: 8002
    targetPort: 8002
  type: ClusterIP
EOF
    
    # Wait for Payment Service to be ready
    kubectl wait --for=condition=ready pod -l app=payment-service -n $NAMESPACE --timeout=300s
    
    log_success "Payment Service deployed"
}

# Deploy Frontend
deploy_frontend() {
    log_info "Deploying Frontend..."
    
    # Create Frontend deployment
    cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: Deployment
metadata:
  name: frontend
  namespace: $NAMESPACE
spec:
  replicas: $REPLICAS
  selector:
    matchLabels:
      app: frontend
  template:
    metadata:
      labels:
        app: frontend
    spec:
      containers:
      - name: frontend
        image: pesabit/frontend:$IMAGE_TAG
        env:
        - name: VITE_API_BASE_URL
          value: "https://api.pesa.co.ke"
        - name: VITE_APP_ENV
          value: "production"
        ports:
        - containerPort: 80
        resources:
          requests:
            memory: "128Mi"
            cpu: "50m"
          limits:
            memory: "512Mi"
            cpu: "200m"
        livenessProbe:
          httpGet:
            path: /
            port: 80
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /
            port: 80
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: frontend
  namespace: $NAMESPACE
spec:
  selector:
    app: frontend
  ports:
  - port: 80
    targetPort: 80
  type: ClusterIP
EOF
    
    # Wait for Frontend to be ready
    kubectl wait --for=condition=ready pod -l app=frontend -n $NAMESPACE --timeout=300s
    
    log_success "Frontend deployed"
}

# Deploy Ingress
deploy_ingress() {
    log_info "Deploying Ingress..."
    
    # Apply ingress configuration
    kubectl apply -f infrastructure/kubernetes/ingress.yaml -n $NAMESPACE
    
    log_success "Ingress deployed"
}

# Deploy Monitoring
deploy_monitoring() {
    log_info "Deploying monitoring stack..."
    
    # Add Prometheus Helm repository
    helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
    helm repo update
    
    # Install Prometheus
    helm install prometheus prometheus-community/kube-prometheus-stack \
        --namespace $NAMESPACE \
        --set prometheus.prometheusSpec.serviceMonitorSelectorNilUsesHelmValues=false \
        --set prometheus.prometheusSpec.podMonitorSelectorNilUsesHelmValues=false \
        --set prometheus.prometheusSpec.ruleSelectorNilUsesHelmValues=false \
        --create-namespace
    
    # Install Grafana
    helm install grafana grafana/grafana \
        --namespace $NAMESPACE \
        --set persistence.enabled=true \
        --set persistence.size=10Gi \
        --set adminPassword=admin \
        --set service.type=LoadBalancer
    
    log_success "Monitoring stack deployed"
}

# Run database migrations
run_migrations() {
    log_info "Running database migrations..."
    
    # Create migration job
    cat <<EOF | kubectl apply -f -
apiVersion: batch/v1
kind: Job
metadata:
  name: pesabit-migrations
  namespace: $NAMESPACE
spec:
  template:
    spec:
      containers:
      - name: migrations
        image: pesabit/api-gateway:$IMAGE_TAG
        command: ["cargo", "run", "--bin", "migrate"]
        env:
        - name: DATABASE_URL
          value: "postgresql://pesabit_prod:\$(DB_PASSWORD)@postgresql:5432/pesabit_prod"
        - name: DB_PASSWORD
          valueFrom:
            secretKeyRef:
              name: pesabit-db-secret
              key: password
        - name: RUST_LOG
          value: "info"
      restartPolicy: Never
  backoffLimit: 3
EOF
    
    # Wait for migration job to complete
    kubectl wait --for=condition=complete job/pesabit-migrations -n $NAMESPACE --timeout=300s
    
    log_success "Database migrations completed"
}

# Verify deployment
verify_deployment() {
    log_info "Verifying deployment..."
    
    # Check all pods are running
    kubectl get pods -n $NAMESPACE
    
    # Check services
    kubectl get services -n $NAMESPACE
    
    # Check ingress
    kubectl get ingress -n $NAMESPACE
    
    # Test health endpoints
    local api_gateway_service=$(kubectl get svc -n $NAMESPACE -l app=api-gateway -o jsonpath='{.items[0].metadata.name}')
    
    if kubectl run test-deployment --image=curlimages/curl:latest \
        --namespace=$NAMESPACE \
        --rm -i --restart=Never \
        --command -- curl -f http://$api_gateway_service:3000/health; then
        log_success "Deployment verification passed"
    else
        log_error "Deployment verification failed"
        exit 1
    fi
}

# Main deployment function
deploy_to_production() {
    log_info "Starting production deployment..."
    
    check_prerequisites
    configure_aws_eks
    create_namespace
    create_secrets
    deploy_postgresql
    deploy_redis
    run_migrations
    deploy_api_gateway
    deploy_user_service
    deploy_payment_service
    deploy_frontend
    deploy_ingress
    deploy_monitoring
    verify_deployment
    
    log_success "Production deployment completed successfully!"
    log_info "PesaBit is now running in production!"
}

# Rollback deployment
rollback_deployment() {
    log_info "Rolling back deployment..."
    
    # Delete all deployments
    kubectl delete deployment --all -n $NAMESPACE
    
    # Delete all services
    kubectl delete service --all -n $NAMESPACE
    
    # Delete all ingress
    kubectl delete ingress --all -n $NAMESPACE
    
    log_success "Deployment rolled back"
}

# Main function
main() {
    case "${1:-deploy}" in
        "deploy")
            deploy_to_production
            ;;
        "rollback")
            rollback_deployment
            ;;
        "verify")
            verify_deployment
            ;;
        *)
            echo "Usage: $0 {deploy|rollback|verify}"
            echo ""
            echo "Commands:"
            echo "  deploy   - Deploy to production"
            echo "  rollback - Rollback deployment"
            echo "  verify   - Verify deployment"
            echo ""
            echo "Examples:"
            echo "  $0 deploy"
            echo "  $0 rollback"
            echo "  $0 verify"
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
