-- Transactions table: Complete record of all money movements
-- This is the core financial ledger - every payment is recorded here

CREATE TABLE transactions (
    -- Primary key
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Which user made this transaction
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- What type of transaction (deposit, withdrawal, Lightning send/receive)
    type transaction_type NOT NULL,
    
    -- Current status (pending → processing → completed/failed)
    status transaction_status NOT NULL DEFAULT 'pending',
    
    -- Amounts in Kenyan Shillings (NULL for pure Lightning transactions)
    amount_kes DECIMAL(15,2),
    
    -- Amounts in Bitcoin satoshis (NULL for pure M-Pesa transactions)
    amount_sats BIGINT,
    
    -- Exchange rate at time of transaction (KES per 100k sats)
    -- Critical for audit and tax reporting
    exchange_rate DECIMAL(15,2),
    
    -- Fees charged in both currencies
    fee_kes DECIMAL(15,2),
    fee_sats BIGINT,
    
    -- M-Pesa transaction reference (e.g., "QL12XYZ789")
    -- Used to match with M-Pesa statements and callbacks
    mpesa_code VARCHAR(50),
    
    -- Lightning invoice (BOLT11 format for requesting payments)
    lightning_invoice TEXT,
    
    -- Lightning payment preimage (proof payment was completed)
    lightning_preimage VARCHAR(64),
    
    -- Flexible JSON field for additional data:
    -- - Recipient Lightning address
    -- - Payment notes/memos
    -- - Device information
    -- - Error details for failed transactions
    metadata JSONB,
    
    -- When transaction was created
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- When transaction was completed (NULL if still pending/processing)
    completed_at TIMESTAMPTZ,
    
    -- Business logic constraints
    CONSTRAINT valid_amounts CHECK (
        (amount_kes IS NOT NULL OR amount_sats IS NOT NULL) -- At least one amount must be set
    ),
    CONSTRAINT valid_fees CHECK (
        (fee_kes IS NULL OR fee_kes >= 0) AND 
        (fee_sats IS NULL OR fee_sats >= 0) -- Fees cannot be negative
    )
);

-- Critical indexes for performance and queries
CREATE INDEX idx_transactions_user_id ON transactions(user_id);
CREATE INDEX idx_transactions_type ON transactions(type);
CREATE INDEX idx_transactions_status ON transactions(status);
CREATE INDEX idx_transactions_created_at ON transactions(created_at);
CREATE INDEX idx_transactions_mpesa_code ON transactions(mpesa_code) WHERE mpesa_code IS NOT NULL;
CREATE INDEX idx_transactions_completed_at ON transactions(completed_at) WHERE completed_at IS NOT NULL;

-- Compound index for user transaction history (most common query)
CREATE INDEX idx_transactions_user_created ON transactions(user_id, created_at DESC);

-- Index on JSON metadata for searching payment notes, recipients, etc.
CREATE INDEX idx_transactions_metadata ON transactions USING gin(metadata);

-- Automatically set completed_at when status changes to completed
CREATE OR REPLACE FUNCTION set_completed_at()
RETURNS TRIGGER AS $$
BEGIN
    -- If status changed to 'completed', set completed_at timestamp
    IF OLD.status != 'completed' AND NEW.status = 'completed' THEN
        NEW.completed_at = NOW();
    END IF;
    
    -- If status changed away from 'completed', clear completed_at
    IF OLD.status = 'completed' AND NEW.status != 'completed' THEN
        NEW.completed_at = NULL;
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER transactions_completed_at
    BEFORE UPDATE ON transactions
    FOR EACH ROW
    EXECUTE FUNCTION set_completed_at();