#! /bin/bash
cargo b --release
sudo ./target/release/trust &
pid=$!
sleep 2
sudo ifconfig utun4 up 192.168.0.1 192.168.0.2
trap "sudo kill $pid" INT TERM
wait $pid

