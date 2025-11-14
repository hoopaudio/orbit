#!/bin/bash

# Start the NeMo Agent Toolkit server for Orbit AI

echo "Starting Orbit AI NeMo Server..."
echo "================================"

# Activate virtual environment
source .venv/bin/activate

# Set GOOGLE_APPLICATION_CREDENTIALS if not already set
if [ -z "$GOOGLE_APPLICATION_CREDENTIALS" ]; then
    export GOOGLE_APPLICATION_CREDENTIALS="src/python/hoop-audio-472016-d46b2b6bf6b4.json"
    echo "Set GOOGLE_APPLICATION_CREDENTIALS to: $GOOGLE_APPLICATION_CREDENTIALS"
fi

# Local memory uses Vertex AI - no external API keys needed
echo "✅ Using local Mem0 with Vertex AI embeddings"

# Start the NeMo server with the Orbit config
# This creates an OpenAI-compatible endpoint at http://localhost:8000/v1/chat/completions
nat serve --config_file src/python/orbit_config.yml

# The server supports streaming responses via SSE (Server-Sent Events)
# You can connect to it with:
# - OpenAI Python client with base_url="http://localhost:8000/v1"
# - curl with SSE support
# - Any OpenAI-compatible client
