-- Create custom types for the database
-- These enums ensure data consistency and prevent invalid values

-- Transaction types (what kind of money movement)
CREATE TYPE transaction_type AS ENUM (
    'deposit_mpesa',     -- User adds money via M-Pesa → gets Bitcoin sats
    'withdrawal_mpesa',  -- User withdraws Bitcoin → gets M-Pesa money  
    'lightning_send',    -- User sends Lightning payment to someone
    'lightning_receive'  -- User receives Lightning payment from someone
);

-- Transaction status (current state of the transaction)
CREATE TYPE transaction_status AS ENUM (
    'pending',     -- Transaction created but not processed yet
    'processing',  -- Currently being processed (M-Pesa confirmation, Lightning routing)
    'completed',   -- Successfully finished
    'failed',      -- Failed due to error (insufficient funds, timeout, etc.)
    'refunded'     -- Failed transaction that was refunded to user
);

-- KYC (Know Your Customer) verification status
CREATE TYPE kyc_status AS ENUM (
    'none',      -- No verification done
    'pending',   -- Documents submitted, waiting for review
    'verified',  -- Identity confirmed successfully
    'rejected'   -- Verification failed or rejected
);

-- KYC tier (determines transaction limits)
CREATE TYPE kyc_tier AS ENUM (
    'tier0',  -- Phone only - 10,000 KES/day limit
    'tier1',  -- ID verified - 100,000 KES/day limit  
    'tier2'   -- Full verification - unlimited
);

-- Enable UUID generation
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_crypto";  -- For cryptographic functions