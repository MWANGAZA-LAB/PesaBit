#!/bin/bash

# PesaBit Backup and Disaster Recovery Script
# This script provides comprehensive backup and recovery capabilities for production

set -euo pipefail

# Configuration
NAMESPACE="pesabit"
BACKUP_DIR="/backups/pesabit"
RETENTION_DAYS=30
S3_BUCKET="pesabit-backups"
AWS_REGION="us-east-1"

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

# Create backup directory
create_backup_dir() {
    log_info "Creating backup directory: $BACKUP_DIR"
    mkdir -p "$BACKUP_DIR"/{database,redis,secrets,configs,logs}
    log_success "Backup directory created"
}

# Backup PostgreSQL database
backup_database() {
    log_info "Starting PostgreSQL backup..."
    
    local timestamp=$(date +"%Y%m%d_%H%M%S")
    local backup_file="$BACKUP_DIR/database/postgres_backup_$timestamp.sql"
    
    # Get PostgreSQL pod name
    local postgres_pod=$(kubectl get pods -n $NAMESPACE -l app.kubernetes.io/name=postgresql -o jsonpath='{.items[0].metadata.name}')
    
    if [ -z "$postgres_pod" ]; then
        log_error "PostgreSQL pod not found"
        return 1
    fi
    
    # Create database backup
    kubectl exec -n $NAMESPACE "$postgres_pod" -- pg_dump -U pesabit -d pesabit > "$backup_file"
    
    # Compress backup
    gzip "$backup_file"
    
    log_success "Database backup completed: ${backup_file}.gz"
    
    # Upload to S3
    if command -v aws &> /dev/null; then
        aws s3 cp "${backup_file}.gz" "s3://$S3_BUCKET/database/"
        log_success "Database backup uploaded to S3"
    fi
}

# Backup Redis data
backup_redis() {
    log_info "Starting Redis backup..."
    
    local timestamp=$(date +"%Y%m%d_%H%M%S")
    local backup_file="$BACKUP_DIR/redis/redis_backup_$timestamp.rdb"
    
    # Get Redis pod name
    local redis_pod=$(kubectl get pods -n $NAMESPACE -l app.kubernetes.io/name=redis -o jsonpath='{.items[0].metadata.name}')
    
    if [ -z "$redis_pod" ]; then
        log_error "Redis pod not found"
        return 1
    fi
    
    # Trigger Redis backup
    kubectl exec -n $NAMESPACE "$redis_pod" -- redis-cli BGSAVE
    
    # Wait for backup to complete
    sleep 10
    
    # Copy backup file
    kubectl cp "$NAMESPACE/$redis_pod:/data/dump.rdb" "$backup_file"
    
    # Compress backup
    gzip "$backup_file"
    
    log_success "Redis backup completed: ${backup_file}.gz"
    
    # Upload to S3
    if command -v aws &> /dev/null; then
        aws s3 cp "${backup_file}.gz" "s3://$S3_BUCKET/redis/"
        log_success "Redis backup uploaded to S3"
    fi
}

# Backup Kubernetes secrets
backup_secrets() {
    log_info "Starting Kubernetes secrets backup..."
    
    local timestamp=$(date +"%Y%m%d_%H%M%S")
    local backup_file="$BACKUP_DIR/secrets/secrets_backup_$timestamp.yaml"
    
    # Export all secrets
    kubectl get secrets -n $NAMESPACE -o yaml > "$backup_file"
    
    # Compress backup
    gzip "$backup_file"
    
    log_success "Secrets backup completed: ${backup_file}.gz"
    
    # Upload to S3
    if command -v aws &> /dev/null; then
        aws s3 cp "${backup_file}.gz" "s3://$S3_BUCKET/secrets/"
        log_success "Secrets backup uploaded to S3"
    fi
}

# Backup application configurations
backup_configs() {
    log_info "Starting application configurations backup..."
    
    local timestamp=$(date +"%Y%m%d_%H%M%S")
    local backup_file="$BACKUP_DIR/configs/configs_backup_$timestamp.tar.gz"
    
    # Create configuration backup
    tar -czf "$backup_file" \
        -C . \
        config/ \
        infrastructure/ \
        migrations/ \
        shared/ \
        services/ \
        frontend/package.json \
        frontend/tsconfig.json \
        Cargo.toml \
        docker-compose.yml \
        Dockerfile
    
    log_success "Configurations backup completed: $backup_file"
    
    # Upload to S3
    if command -v aws &> /dev/null; then
        aws s3 cp "$backup_file" "s3://$S3_BUCKET/configs/"
        log_success "Configurations backup uploaded to S3"
    fi
}

# Backup application logs
backup_logs() {
    log_info "Starting application logs backup..."
    
    local timestamp=$(date +"%Y%m%d_%H%M%S")
    local backup_file="$BACKUP_DIR/logs/logs_backup_$timestamp.tar.gz"
    
    # Create logs directory
    mkdir -p "$BACKUP_DIR/logs/temp"
    
    # Get logs from all pods
    for pod in $(kubectl get pods -n $NAMESPACE -o jsonpath='{.items[*].metadata.name}'); do
        kubectl logs -n $NAMESPACE "$pod" --previous > "$BACKUP_DIR/logs/temp/${pod}_previous.log" 2>/dev/null || true
        kubectl logs -n $NAMESPACE "$pod" > "$BACKUP_DIR/logs/temp/${pod}_current.log" 2>/dev/null || true
    done
    
    # Compress logs
    tar -czf "$backup_file" -C "$BACKUP_DIR/logs/temp" .
    rm -rf "$BACKUP_DIR/logs/temp"
    
    log_success "Logs backup completed: $backup_file"
    
    # Upload to S3
    if command -v aws &> /dev/null; then
        aws s3 cp "$backup_file" "s3://$S3_BUCKET/logs/"
        log_success "Logs backup uploaded to S3"
    fi
}

# Cleanup old backups
cleanup_old_backups() {
    log_info "Cleaning up old backups (older than $RETENTION_DAYS days)..."
    
    find "$BACKUP_DIR" -type f -name "*.gz" -mtime +$RETENTION_DAYS -delete
    
    log_success "Old backups cleaned up"
}

# Restore database from backup
restore_database() {
    local backup_file="$1"
    
    if [ ! -f "$backup_file" ]; then
        log_error "Backup file not found: $backup_file"
        return 1
    fi
    
    log_info "Restoring database from: $backup_file"
    
    # Get PostgreSQL pod name
    local postgres_pod=$(kubectl get pods -n $NAMESPACE -l app.kubernetes.io/name=postgresql -o jsonpath='{.items[0].metadata.name}')
    
    if [ -z "$postgres_pod" ]; then
        log_error "PostgreSQL pod not found"
        return 1
    fi
    
    # Decompress if needed
    if [[ "$backup_file" == *.gz ]]; then
        gunzip -c "$backup_file" | kubectl exec -i -n $NAMESPACE "$postgres_pod" -- psql -U pesabit -d pesabit
    else
        kubectl exec -i -n $NAMESPACE "$postgres_pod" -- psql -U pesabit -d pesabit < "$backup_file"
    fi
    
    log_success "Database restored successfully"
}

# Restore Redis from backup
restore_redis() {
    local backup_file="$1"
    
    if [ ! -f "$backup_file" ]; then
        log_error "Backup file not found: $backup_file"
        return 1
    fi
    
    log_info "Restoring Redis from: $backup_file"
    
    # Get Redis pod name
    local redis_pod=$(kubectl get pods -n $NAMESPACE -l app.kubernetes.io/name=redis -o jsonpath='{.items[0].metadata.name}')
    
    if [ -z "$redis_pod" ]; then
        log_error "Redis pod not found"
        return 1
    fi
    
    # Stop Redis to restore
    kubectl exec -n $NAMESPACE "$redis_pod" -- redis-cli SHUTDOWN SAVE
    
    # Wait for pod to restart
    kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=redis -n $NAMESPACE --timeout=300s
    
    # Get new pod name
    redis_pod=$(kubectl get pods -n $NAMESPACE -l app.kubernetes.io/name=redis -o jsonpath='{.items[0].metadata.name}')
    
    # Copy backup file
    kubectl cp "$backup_file" "$NAMESPACE/$redis_pod:/data/dump.rdb"
    
    # Restart Redis
    kubectl exec -n $NAMESPACE "$redis_pod" -- redis-cli SHUTDOWN SAVE
    
    log_success "Redis restored successfully"
}

# Download backup from S3
download_backup_from_s3() {
    local backup_type="$1"
    local backup_name="$2"
    
    if ! command -v aws &> /dev/null; then
        log_error "AWS CLI not installed"
        return 1
    fi
    
    log_info "Downloading $backup_type backup from S3: $backup_name"
    
    aws s3 cp "s3://$S3_BUCKET/$backup_type/$backup_name" "$BACKUP_DIR/$backup_type/"
    
    log_success "Backup downloaded from S3"
}

# Test backup integrity
test_backup_integrity() {
    log_info "Testing backup integrity..."
    
    # Test database backup
    local db_backup=$(find "$BACKUP_DIR/database" -name "*.gz" -type f | head -1)
    if [ -n "$db_backup" ]; then
        if gunzip -t "$db_backup"; then
            log_success "Database backup integrity check passed"
        else
            log_error "Database backup integrity check failed"
            return 1
        fi
    fi
    
    # Test Redis backup
    local redis_backup=$(find "$BACKUP_DIR/redis" -name "*.gz" -type f | head -1)
    if [ -n "$redis_backup" ]; then
        if gunzip -t "$redis_backup"; then
            log_success "Redis backup integrity check passed"
        else
            log_error "Redis backup integrity check failed"
            return 1
        fi
    fi
    
    log_success "All backup integrity checks passed"
}

# Main backup function
perform_backup() {
    log_info "Starting comprehensive backup process..."
    
    create_backup_dir
    backup_database
    backup_redis
    backup_secrets
    backup_configs
    backup_logs
    cleanup_old_backups
    test_backup_integrity
    
    log_success "Backup process completed successfully!"
}

# Main restore function
perform_restore() {
    local backup_type="$1"
    local backup_file="$2"
    
    log_info "Starting restore process for $backup_type..."
    
    case "$backup_type" in
        "database")
            restore_database "$backup_file"
            ;;
        "redis")
            restore_redis "$backup_file"
            ;;
        *)
            log_error "Unknown backup type: $backup_type"
            return 1
            ;;
    esac
    
    log_success "Restore process completed successfully!"
}

# Disaster recovery function
disaster_recovery() {
    log_info "Starting disaster recovery process..."
    
    # Check if cluster is accessible
    if ! kubectl cluster-info &> /dev/null; then
        log_error "Cannot access Kubernetes cluster"
        return 1
    fi
    
    # Check if namespace exists
    if ! kubectl get namespace $NAMESPACE &> /dev/null; then
        log_warning "Namespace $NAMESPACE does not exist, creating..."
        kubectl create namespace $NAMESPACE
    fi
    
    # Download latest backups from S3
    log_info "Downloading latest backups from S3..."
    
    # Get latest database backup
    local latest_db_backup=$(aws s3 ls "s3://$S3_BUCKET/database/" --recursive | sort | tail -1 | awk '{print $4}')
    if [ -n "$latest_db_backup" ]; then
        download_backup_from_s3 "database" "$latest_db_backup"
    fi
    
    # Get latest Redis backup
    local latest_redis_backup=$(aws s3 ls "s3://$S3_BUCKET/redis/" --recursive | sort | tail -1 | awk '{print $4}')
    if [ -n "$latest_redis_backup" ]; then
        download_backup_from_s3 "redis" "$latest_redis_backup"
    fi
    
    # Get latest configurations backup
    local latest_config_backup=$(aws s3 ls "s3://$S3_BUCKET/configs/" --recursive | sort | tail -1 | awk '{print $4}')
    if [ -n "$latest_config_backup" ]; then
        download_backup_from_s3 "configs" "$latest_config_backup"
    fi
    
    # Restore configurations
    if [ -n "$latest_config_backup" ]; then
        log_info "Restoring configurations..."
        tar -xzf "$BACKUP_DIR/configs/$latest_config_backup" -C /tmp/
        log_success "Configurations restored"
    fi
    
    # Deploy infrastructure
    log_info "Deploying infrastructure..."
    kubectl apply -f infrastructure/kubernetes/ -n $NAMESPACE
    
    # Wait for services to be ready
    kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=postgresql -n $NAMESPACE --timeout=300s
    kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=redis -n $NAMESPACE --timeout=300s
    
    # Restore data
    if [ -n "$latest_db_backup" ]; then
        restore_database "$BACKUP_DIR/database/$latest_db_backup"
    fi
    
    if [ -n "$latest_redis_backup" ]; then
        restore_redis "$BACKUP_DIR/redis/$latest_redis_backup"
    fi
    
    # Deploy application
    log_info "Deploying application..."
    kubectl apply -f infrastructure/kubernetes/ -n $NAMESPACE
    
    # Wait for application to be ready
    kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=pesabit-api-gateway -n $NAMESPACE --timeout=300s
    
    # Verify recovery
    log_info "Verifying disaster recovery..."
    
    # Test health endpoint
    local api_gateway_service=$(kubectl get svc -n $NAMESPACE -l app.kubernetes.io/name=pesabit-api-gateway -o jsonpath='{.items[0].metadata.name}')
    
    if kubectl run test-recovery --image=curlimages/curl:latest \
        --namespace=$NAMESPACE \
        --rm -i --restart=Never \
        --command -- curl -f http://$api_gateway_service:3000/health; then
        log_success "Disaster recovery verification passed"
    else
        log_error "Disaster recovery verification failed"
        return 1
    fi
    
    log_success "Disaster recovery completed successfully!"
}

# Main function
main() {
    case "${1:-backup}" in
        "backup")
            perform_backup
            ;;
        "restore")
            if [ $# -lt 3 ]; then
                log_error "Usage: $0 restore <backup_type> <backup_file>"
                exit 1
            fi
            perform_restore "$2" "$3"
            ;;
        "disaster-recovery")
            disaster_recovery
            ;;
        "test-integrity")
            test_backup_integrity
            ;;
        *)
            echo "Usage: $0 {backup|restore|disaster-recovery|test-integrity}"
            echo ""
            echo "Commands:"
            echo "  backup                 - Perform comprehensive backup"
            echo "  restore <type> <file>  - Restore from backup"
            echo "  disaster-recovery     - Perform disaster recovery"
            echo "  test-integrity        - Test backup integrity"
            echo ""
            echo "Examples:"
            echo "  $0 backup"
            echo "  $0 restore database /backups/pesabit/database/postgres_backup_20240101_120000.sql.gz"
            echo "  $0 disaster-recovery"
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
