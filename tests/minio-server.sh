#!/bin/bash
export MINIO_ROOT_USER=minio-access-key-test
export MINIO_ROOT_PASSWORD=minio-secret-key-test
export MINIO_DOMAIN=localhost

minio server --address 0.0.0.0:9022 --console-address :9023 disk