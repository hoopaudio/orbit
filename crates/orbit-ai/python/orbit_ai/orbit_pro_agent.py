"""
Orbit Pro Agent - Advanced AI assistant powered by LangGraph.
Supports multiple providers including Vertex AI, OpenAI, and OpenRouter.
"""

from typing import Optional
from orbit_ai.langgraph_agent import LangGraphAgent


class OrbitProAgent(LangGraphAgent):
    """Orbit Pro Agent - Professional AI assistant with multi-provider support"""
    pass


# Factory function to create agent instancae
def create_agent(api_key: Optional[str] = None) -> OrbitProAgent:
    """Create a new instance of the Orbit Pro Agent."""
    return OrbitProAgent(api_key)