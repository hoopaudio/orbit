"""
Orbit Pro Agent - Advanced AI assistant powered by LangChain.
Supports multiple providers including Vertex AI, OpenAI, and OpenRouter.
"""

from orbit_ai.langchain_bot import LangChainBot


class OrbitProAgent(LangChainBot):
    """Orbit Pro Agent - Professional AI assistant with multi-provider support"""
    pass


# Factory function to create agent instance
def create_agent(api_key: str = None) -> OrbitProAgent:
    """Create a new instance of the Orbit Pro Agent."""
    return OrbitProAgent(api_key)