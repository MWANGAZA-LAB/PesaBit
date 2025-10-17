-- Wallets table: User balances in different currencies
-- Each user has exactly one wallet that tracks their money

CREATE TABLE wallets (
    -- Primary key
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Links to user (one wallet per user)
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Confirmed Bitcoin balance in satoshis (1 BTC = 100,000,000 sats)
    -- This is the money user can spend immediately
    balance_sats BIGINT NOT NULL DEFAULT 0,
    
    -- Pending M-Pesa balance in Kenyan Shillings (before conversion to Bitcoin)
    -- Temporary balance while M-Pesa payment is being processed
    balance_kes DECIMAL(15,2) NOT NULL DEFAULT 0,
    
    -- Unconfirmed Lightning payments (waiting for confirmation)
    -- Lightning payments that are routing but not confirmed yet
    pending_balance_sats BIGINT NOT NULL DEFAULT 0,
    
    -- When wallet was last updated (important for financial reconciliation)
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Ensure each user has exactly one wallet
    CONSTRAINT unique_user_wallet UNIQUE(user_id),
    
    -- Prevent negative balances (important financial safeguard)
    CONSTRAINT positive_balance_sats CHECK (balance_sats >= 0),
    CONSTRAINT positive_balance_kes CHECK (balance_kes >= 0),
    CONSTRAINT positive_pending_sats CHECK (pending_balance_sats >= 0)
);

-- Indexes for fast balance lookups
CREATE INDEX idx_wallets_user_id ON wallets(user_id);
CREATE INDEX idx_wallets_updated_at ON wallets(updated_at);

-- Update timestamp when wallet balance changes
CREATE TRIGGER wallets_updated_at
    BEFORE UPDATE ON wallets
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();