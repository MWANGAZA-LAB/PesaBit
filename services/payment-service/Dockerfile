FROM pesabit-base as development

# Set service-specific working directory
WORKDIR /app/services/payment-service

# Expose payment service port
EXPOSE 8002

# Development command with hot reload
CMD ["cargo", "watch", "-x", "run"]

# Production stage
FROM pesabit-base as production

# Copy the built payment service binary
COPY --from=builder /app/target/release/payment-service ./bin/

# Expose payment service port
EXPOSE 8002

# Run the payment service
CMD ["./bin/payment-service"]