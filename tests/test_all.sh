#!/bin/bash

export MINIO_HOST="localhost:9022"
export MINIO_ACCESS_KEY=minio-access-key-test
export MINIO_SECRET_KEY=minio-secret-key-test

export virtual_hosted_style=false
export multi_chunked=false
cargo test --features "fs-tokio","ext"


export virtual_hosted_style=true
export multi_chunked=true
cargo test --features "fs-tokio","ext"