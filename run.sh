#!/bin/sh
ssh root@$PIIP 'pkill drive-pi ; rm -rf /app/src ; rm /app/Cargo.toml'
scp Cargo.toml root@$PIIP:/app/Cargo.toml
scp -r src root@$PIIP:/app/src
ssh root@$PIIP 'cd /app ; cargo run'
