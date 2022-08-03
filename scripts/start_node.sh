#!/bin/bash
echo "staring node for $1 ..."
./target/release/node-template $1 --chain ./scripts/local.chain --tmp --rpc-cors all --log runtime::dpos

