#!/bin/sh
ssh root@192.168.1.196 'pkill drive-pi ; rm -rf /app/src ; mkdir /app/Cargo.toml'
scp Cargo.toml root@192.168.1.196:/app/Cargo.toml
scp -r src root@192.168.1.196:/app/src
ssh root@192.168.1.196 'cd /app ; cargo run'
