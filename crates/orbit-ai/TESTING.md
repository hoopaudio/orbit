# Testing Guide for orbit-ai

## Rust Tests

### Running all tests (excluding API tests)
```bash
cargo test
```

### Running API tests (requires quota)
```bash
cargo test -- --ignored
```

### Running a specific test
```bash
cargo test test_ask_orbit -- --ignored
```

## Python Tests

### Setup
```bash
# Ensure you're in the virtual environment
source .venv/bin/activate

# Install test dependencies (if not already installed)
~/.local/bin/uv pip install pytest pytest-asyncio
```

### Running Python tests
```bash
# Run all tests
pytest tests/

# Run with verbose output
pytest tests/ -v

# Run a specific test file
pytest tests/test_orbit_ai.py

# Run a specific test
pytest tests/test_orbit_ai.py::test_get_system_prompt
```

## Test Structure

### Rust Tests
- Located in `src/langchain_test.rs`
- Tests marked with `#[ignore]` require API quota and won't run by default
- Includes tests for:
  - Basic chat functionality
  - Image-based chat
  - Screenshot tool integration

### Python Tests
- Located in `tests/` directory
- Tests the PyO3 bindings and Python API
- Includes tests for:
  - Module imports
  - Getting system prompt
  - Setting API key
  - Module contents verification
  - Class structure validation

## Notes

- The API tests are ignored by default to avoid rate limiting
- The testdata directory contains test images for the Rust tests
- Python tests don't make actual API calls, only test the bindings