# Build release binary
build:
    cargo build --release

# Restart the service (rebuild and reload)
restart: build
    launchctl kickstart -k gui/$(id -u)/com.syndicate-json-canvas

# View service status
status:
    launchctl list | grep syndicate || echo "Service not running"

# Run in foreground (for debugging)
run:
    cargo run --release
