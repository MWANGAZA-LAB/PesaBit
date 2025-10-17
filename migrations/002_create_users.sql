-- Users table: Core user accounts and authentication
-- This stores essential user information and login credentials

CREATE TABLE users (
    -- Primary key (UUID for security, not predictable like integers)
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Phone number in E.164 format (e.g., +254712345678)
    -- This is the user's unique identifier, like an email address
    phone_number VARCHAR(15) UNIQUE NOT NULL,
    
    -- Hashed PIN using Argon2id (secure against GPU attacks)
    -- Never store PINs in plain text for security
    pin_hash VARCHAR(128) NOT NULL,
    
    -- Lightning address username (e.g., "john" for john@pesa.co.ke)
    -- This allows users to receive Bitcoin payments via their address
    lightning_username VARCHAR(50) UNIQUE NOT NULL,
    
    -- User's full name (optional, for KYC and personalization)
    full_name VARCHAR(100),
    
    -- KYC verification status and tier (affects transaction limits)
    kyc_status kyc_status NOT NULL DEFAULT 'none',
    kyc_tier kyc_tier NOT NULL DEFAULT 'tier0',
    
    -- Timestamps for tracking when account was created/updated
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for fast lookups (critical for performance)
CREATE INDEX idx_users_phone ON users(phone_number);
CREATE INDEX idx_users_lightning_username ON users(lightning_username);
CREATE INDEX idx_users_kyc_status ON users(kyc_status);
CREATE INDEX idx_users_created_at ON users(created_at);

-- Update the updated_at timestamp automatically when row changes
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();