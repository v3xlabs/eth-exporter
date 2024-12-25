# =====================================
# Stage 1: User Setup
# Creates a non-root user for running the application securely
# =====================================
FROM alpine:latest as usersetup
RUN addgroup -g 10001 -S dockergrp && \
    adduser -u 10001 -S dockeruser -G dockergrp

# =====================================
# Stage 2: Final Image
# Minimal scratch-based image containing only the essential components
# =====================================
FROM scratch

# Security: Copy user information and switch to non-root user
COPY --from=usersetup /etc/passwd /etc/passwd
USER dockeruser

# Application Configuration
# -----------------------
# RUST_LOG: Logging configuration (default: error,rpc-to-prom=info)
ENV RUST_LOG="error,rpc-to-prom=info"

# Binary Installation
# -----------------
# BINARY_PATH: Path to the pre-compiled binary from GitHub Actions
ARG BINARY_PATH
COPY ${BINARY_PATH} /rpc-to-prom

# Network Configuration
# -------------------
EXPOSE 3000

# Application Startup
# -----------------
# Using exec form of CMD as there is no shell in scratch image
CMD ["/rpc-to-prom"]
