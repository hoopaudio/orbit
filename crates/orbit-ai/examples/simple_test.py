#!/usr/bin/env python3
"""
Simple test of orbit-ai Python bindings

Usage:
    cd crates/orbit-ai
    source .venv/bin/activate
    python examples/simple_test.py
"""

import sys
from pathlib import Path

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

try:
    import orbit_ai
    print("✅ Successfully imported orbit_ai module")
    print(f"   Module location: {orbit_ai.__file__}")
    print()

    # Test getting the system prompt
    prompt = orbit_ai.get_system_prompt()
    print("✅ System prompt retrieved successfully")
    print(f"   Length: {len(prompt)} characters")
    print(f"   Preview: {prompt[:100]}...")
    print()

    # Test setting API key
    orbit_ai.set_openrouter_api_key("test-key-123")
    print("✅ API key setter works")
    print()

    # List available items
    print("📦 Available items in module:")
    for item in sorted(dir(orbit_ai)):
        if not item.startswith('_'):
            print(f"   - {item}")

except ImportError as e:
    print(f"❌ Error importing orbit_ai: {e}")
    print("\n📝 To fix this, run:")
    print("   cd crates/orbit-ai")
    print("   source .venv/bin/activate")
    print("   maturin develop --features python")
    sys.exit(1)
except Exception as e:
    print(f"❌ Unexpected error: {e}")
    sys.exit(1)