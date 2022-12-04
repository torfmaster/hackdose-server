#!/bin/bash
source .env
cargo build --target=$TARGET --release
sshpass -p $PI_PASSWORD ssh -p $PI_SSH_PORT $PI_USER@$PI_HOST 'systemctl stop hackdose.service'
sshpass -p $PI_PASSWORD  scp -C -P$PI_SSH_PORT ../target/$TARGET/release/hackdose $PI_USER@$PI_HOST:/root/ 
sshpass -p $PI_PASSWORD  scp -C -P$PI_SSH_PORT config.yaml $PI_USER@$PI_HOST:/root/
sshpass -p $PI_PASSWORD  ssh -p $PI_SSH_PORT $PI_USER@$PI_HOST 'systemctl start hackdose.service'
