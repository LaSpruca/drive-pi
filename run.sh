cargo build --target armv7-unknown-linux-gnueabihf
scp target/armv7-unknown-linux-gnueabi/debug/drive-pi root@192.168.1.196:/app/drive-pi
ssh root@192.168.1.196 'chmod +x /app/drive-pi | chown root /app/drive-pi | /app/drive-pi'
