#!/bin/sh

cargo build --release --target x86_64-unknown-linux-musl
cp ~/.cargo-target/x86_64-unknown-linux-musl/release/lttpoll-com .
docker build -t registry.olback.dev/olback/lttpoll .
docker push registry.olback.dev/olback/lttpoll
