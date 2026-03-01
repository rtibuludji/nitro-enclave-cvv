
# RUN as SERVICE
## 1. Build Release Binary
```bash
cargo build --release
```

## 2.Copy to EC2 Instance
```bash
# From your dev machine
scp target/release/nitro-cvv-secret ec2-user@your-ec2-ip:/home/ec2-user/

# SSH to EC2
ssh ec2-user@your-ec2-ip

# Move to system location
sudo mv nitro-cvv-secret /usr/local/bin/
sudo chmod +x /usr/local/bin/nitro-cvv-secret
```
## 3. Create Systemd Service
```bash
# Create service file
sudo nano /etc/systemd/system/nitro-cvv-secret.service
```

```ini
[Unit]
Description=Nitro CVV Secret Service
After=network.target

[Service]
Type=simple
User=ec2-user
WorkingDirectory=/home/ec2-user

# Binary location
ExecStart=/usr/local/bin/nitro-cvv-secret

# Environment variables
Environment="SECRET_PORT=3000"
Environment="KMS_KEY_ID=alias/nitro-cvv-key"
Environment="RUST_LOG=info"

# Restart policy
Restart=on-failure
RestartSec=5s

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=nitro-cvv-secret

[Install]
WantedBy=multi-user.target
```

## 4. Enable and Start Service
```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable service (start on boot)
sudo systemctl enable nitro-cvv-secret

# Start service
sudo systemctl start nitro-cvv-secret

# Check status
sudo systemctl status nitro-cvv-secret

# View logs
sudo journalctl -u nitro-cvv-secret -f
```

# RUN as Background Process with NOHUP
```bash
# Run in background
nohup ./nitro-cvv-secret > server.log 2>&1 &

# Save PID
echo $! > server.pid

# Check logs
tail -f server.log

# Stop later
kill $(cat server.pid)
```