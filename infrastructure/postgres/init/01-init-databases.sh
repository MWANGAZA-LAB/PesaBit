#!/bin/bash
set -e

# Create test database for running tests
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    CREATE DATABASE pesabit_test;
    GRANT ALL PRIVILEGES ON DATABASE pesabit_test TO pesabit;
EOSQL

echo "PostgreSQL initialization completed!"
echo "Created databases:"
echo "  - pesabit (main database)"
echo "  - pesabit_test (test database)"