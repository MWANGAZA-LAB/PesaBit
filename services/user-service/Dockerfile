FROM pesabit-base as development

# Set service-specific working directory
WORKDIR /app/services/user-service

# Expose user service port
EXPOSE 8001

# Development command with hot reload
CMD ["cargo", "watch", "-x", "run"]

# Production stage
FROM pesabit-base as production

# Copy the built user service binary
COPY --from=builder /app/target/release/user-service ./bin/

# Expose user service port
EXPOSE 8001

# Run the user service
CMD ["./bin/user-service"]