# Build release binary
build:
    cargo build --release

# Install the service (first-time setup)
install: build
    cp com.syndicate-json-canvas.plist ~/Library/LaunchAgents/
    launchctl load ~/Library/LaunchAgents/com.syndicate-json-canvas.plist

# Restart the service (rebuild and reload)
restart: build
    launchctl kickstart -k gui/$(id -u)/com.syndicate-json-canvas

# View service status
status:
    launchctl list | grep syndicate || echo "Service not running"

# Run in foreground (for debugging)
run:
    cargo run --release
