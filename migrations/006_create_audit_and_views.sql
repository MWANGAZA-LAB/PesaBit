-- Audit log table: Immutable record of all important actions
-- Critical for compliance, security monitoring, and debugging

CREATE TABLE audit_logs (
    -- Primary key (auto-incrementing for chronological order)
    id BIGSERIAL PRIMARY KEY,
    
    -- Which user performed the action (NULL for system actions)
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    
    -- Type of action performed
    action VARCHAR(100) NOT NULL,
    
    -- Which entity was affected (user, transaction, wallet, etc.)
    entity_type VARCHAR(50) NOT NULL,
    
    -- ID of the affected entity
    entity_id VARCHAR(100) NOT NULL,
    
    -- Full details of what changed (before and after state)
    details JSONB NOT NULL,
    
    -- IP address of the user (for security analysis)
    ip_address INET,
    
    -- User agent (browser/app information)
    user_agent TEXT,
    
    -- When action occurred (immutable timestamp)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for audit queries
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);
CREATE INDEX idx_audit_logs_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at DESC);

-- Prevent modification of audit logs (immutable for compliance)
CREATE OR REPLACE FUNCTION prevent_audit_modification()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'Audit logs cannot be modified';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER prevent_audit_update
    BEFORE UPDATE OR DELETE ON audit_logs
    FOR EACH ROW
    EXECUTE FUNCTION prevent_audit_modification();

-- Function to add audit log entries (used by application code)
CREATE OR REPLACE FUNCTION add_audit_log(
    p_user_id UUID,
    p_action VARCHAR(100),
    p_entity_type VARCHAR(50),
    p_entity_id VARCHAR(100),
    p_details JSONB,
    p_ip_address INET DEFAULT NULL,
    p_user_agent TEXT DEFAULT NULL
)
RETURNS void AS $$
BEGIN
    INSERT INTO audit_logs (
        user_id, action, entity_type, entity_id, details, ip_address, user_agent
    ) VALUES (
        p_user_id, p_action, p_entity_type, p_entity_id, p_details, p_ip_address, p_user_agent
    );
END;
$$ LANGUAGE plpgsql;

-- Create views for common queries
-- These make it easier to query the data without complex JOINs

-- User dashboard: Recent transactions with all details
CREATE VIEW user_transaction_history AS
SELECT 
    t.id,
    t.user_id,
    t.type,
    t.status,
    t.amount_kes,
    t.amount_sats,
    t.fee_kes,
    t.fee_sats,
    t.mpesa_code,
    t.metadata,
    t.created_at,
    t.completed_at,
    u.phone_number,
    u.lightning_username
FROM transactions t
JOIN users u ON t.user_id = u.id
ORDER BY t.created_at DESC;

-- User wallet summary with transaction counts
CREATE VIEW user_wallet_summary AS
SELECT 
    u.id as user_id,
    u.phone_number,
    u.lightning_username,
    u.full_name,
    u.kyc_tier,
    w.balance_sats,
    w.balance_kes,
    w.pending_balance_sats,
    w.updated_at as last_activity,
    COUNT(t.id) as total_transactions,
    COUNT(CASE WHEN t.status = 'completed' THEN 1 END) as completed_transactions
FROM users u
LEFT JOIN wallets w ON u.id = w.user_id
LEFT JOIN transactions t ON u.id = t.user_id
GROUP BY u.id, w.balance_sats, w.balance_kes, w.pending_balance_sats, w.updated_at;

-- Daily transaction metrics for monitoring
CREATE VIEW daily_transaction_metrics AS
SELECT 
    DATE(created_at) as date,
    type,
    status,
    COUNT(*) as transaction_count,
    COALESCE(SUM(amount_kes), 0) as total_kes,
    COALESCE(SUM(amount_sats), 0) as total_sats,
    COALESCE(SUM(fee_kes), 0) as total_fees_kes,
    COALESCE(SUM(fee_sats), 0) as total_fees_sats
FROM transactions
WHERE created_at >= CURRENT_DATE - INTERVAL '90 days'
GROUP BY DATE(created_at), type, status
ORDER BY date DESC, type, status;