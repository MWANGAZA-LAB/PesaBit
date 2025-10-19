FROM pesabit-base as development

# Set service-specific working directory
WORKDIR /app/services/api-gateway

# Expose API gateway port
EXPOSE 3000

# Development command with hot reload
CMD ["cargo", "watch", "-x", "run"]

# Production stage
FROM pesabit-base as production

# Copy the built API gateway binary
COPY --from=builder /app/target/release/api-gateway ./bin/

# Expose API gateway port
EXPOSE 3000

# Run the API gateway
CMD ["./bin/api-gateway"]