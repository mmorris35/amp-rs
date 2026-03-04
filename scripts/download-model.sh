#!/bin/bash
set -euo pipefail

MODEL_DIR="${1:-models}"
MODEL_NAME="sentence-transformers/all-MiniLM-L6-v2"

mkdir -p "$MODEL_DIR"

echo "Downloading all-MiniLM-L6-v2 ONNX model..."

# Download model.onnx
curl -L "https://huggingface.co/${MODEL_NAME}/resolve/main/onnx/model.onnx" \
    -o "${MODEL_DIR}/model.onnx"

# Download tokenizer.json
curl -L "https://huggingface.co/${MODEL_NAME}/resolve/main/tokenizer.json" \
    -o "${MODEL_DIR}/tokenizer.json"

echo "Model downloaded to ${MODEL_DIR}/"
ls -la "${MODEL_DIR}/"
