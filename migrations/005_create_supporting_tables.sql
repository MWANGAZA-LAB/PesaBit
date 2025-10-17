-- Exchange rates table: Historical Bitcoin/KES price data
-- Used for converting between Bitcoin and Kenyan Shillings

CREATE TABLE exchange_rates (
    -- Primary key (auto-incrementing for time series data)
    id SERIAL PRIMARY KEY,
    
    -- Bitcoin price in Kenyan Shillings (e.g., 5,300,000 KES per BTC)
    btc_kes DECIMAL(15,2) NOT NULL,
    
    -- Data source (blockchain.info, binance, coinbase, etc.)
    -- Important for tracking which API provided the rate
    source VARCHAR(50) NOT NULL,
    
    -- When this rate was recorded
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Ensure positive prices only
    CONSTRAINT positive_btc_kes CHECK (btc_kes > 0)
);

-- Indexes for fast rate lookups
CREATE INDEX idx_exchange_rates_created_at ON exchange_rates(created_at DESC);
CREATE INDEX idx_exchange_rates_source ON exchange_rates(source);

-- Sessions table: Manage user login sessions and refresh tokens
-- Supports secure authentication with token refresh

CREATE TABLE sessions (
    -- Primary key
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Which user this session belongs to
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Hashed refresh token (never store tokens in plain text)
    refresh_token_hash VARCHAR(128) NOT NULL,
    
    -- When this session expires
    expires_at TIMESTAMPTZ NOT NULL,
    
    -- Device fingerprint (browser, mobile app, etc.)
    -- Used for security monitoring and device identification
    device_fingerprint JSONB,
    
    -- When session was created
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Ensure each user can only have one active session (single device login)
    CONSTRAINT unique_user_session UNIQUE(user_id)
);

-- Indexes for session management
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);

-- OTP codes table: Store SMS verification codes temporarily
-- Used for phone number verification during registration

CREATE TABLE otp_codes (
    -- Primary key
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Phone number this code was sent to
    phone_number VARCHAR(15) NOT NULL,
    
    -- The 6-digit verification code (hashed for security)
    code_hash VARCHAR(128) NOT NULL,
    
    -- When this code expires (typically 5-10 minutes)
    expires_at TIMESTAMPTZ NOT NULL,
    
    -- Whether code has been used (prevent replay attacks)
    used BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- How many times user tried to verify this code
    attempts INTEGER NOT NULL DEFAULT 0,
    
    -- When code was generated
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Limit verification attempts
    CONSTRAINT max_attempts CHECK (attempts <= 5)
);

-- Indexes for OTP verification
CREATE INDEX idx_otp_codes_phone ON otp_codes(phone_number);
CREATE INDEX idx_otp_codes_expires_at ON otp_codes(expires_at);

-- Clean up expired OTP codes automatically (runs daily)
-- This prevents the table from growing indefinitely
CREATE OR REPLACE FUNCTION cleanup_expired_otp_codes()
RETURNS void AS $$
BEGIN
    DELETE FROM otp_codes WHERE expires_at < NOW() - INTERVAL '1 day';
END;
$$ LANGUAGE plpgsql;