"""
Local memory implementation using Mem0's local Memory class.
This avoids the need for Mem0 cloud API keys.
"""

import os
from typing import Any, Dict, List, Optional
from mem0 import Memory
from mem0.configs.base import MemoryConfig, VectorStoreConfig, LlmConfig, EmbedderConfig
from mem0.configs.vector_stores.qdrant import QdrantConfig

from nat.data_models.memory import MemoryBaseConfig
from nat.memory.interfaces import MemoryEditor
from nat.memory.models import MemoryItem


class LocalMem0Config(MemoryBaseConfig, name="local_mem0"):
    """Configuration for local Mem0 memory."""
    qdrant_path: str = "/tmp/qdrant_orbit"
    history_db_path: str = "/tmp/orbit_mem0_history.db"


class LocalMem0Editor(MemoryEditor):
    """Memory editor using local Mem0."""

    def __init__(self, config: LocalMem0Config):
        # Configure Mem0 for local usage
        vector_store_config = VectorStoreConfig(
            provider="qdrant",
            config={
                "collection_name": "orbit_memory",
                "embedding_model_dims": 256,  # Match Vertex AI embeddings
                "path": config.qdrant_path,
                "on_disk": True
            }
        )

        # Configure to use LiteLLM for LLM (supports Vertex AI)
        llm_config = LlmConfig(provider="litellm", config={
            "model": "vertex_ai/gemini-2.5-flash"
        })

        embedder_config = EmbedderConfig(provider="vertexai", config={
            "model": "text-embedding-004",
            "embedding_dims": 256,
            "memory_add_embedding_type": "RETRIEVAL_DOCUMENT"
        })

        memory_config = MemoryConfig(
            vector_store=vector_store_config,
            llm=llm_config,
            embedder=embedder_config,
            history_db_path=config.history_db_path
        )

        self.memory = Memory(config=memory_config)

    async def add_items(self, items: List[MemoryItem]) -> None:
        """Add memory items."""
        for item in items:
            # Extract session_id from metadata if available for working memory
            session_id = item.metadata.get("session_id") if item.metadata else None

            if item.conversation:
                # Convert conversation to messages format
                messages = item.conversation
                kwargs = {
                    "messages": messages,
                    "user_id": item.user_id,
                    "metadata": item.metadata
                }
                # Add run_id for session memory if available
                if session_id:
                    kwargs["run_id"] = session_id

                self.memory.add(**kwargs)
            elif item.memory:
                # Add direct memory text
                kwargs = {
                    "messages": item.memory,
                    "user_id": item.user_id,
                    "metadata": item.metadata
                }
                # Add run_id for session memory if available
                if session_id:
                    kwargs["run_id"] = session_id

                self.memory.add(**kwargs)

    async def search(self, query: str, top_k: int = 5, **kwargs) -> List[MemoryItem]:
        """Search for relevant memories."""
        user_id = kwargs.get("user_id")
        session_id = kwargs.get("session_id")

        search_kwargs = {
            "query": query,
            "user_id": user_id,
            "limit": top_k
        }

        # Add run_id for session-specific search if available
        if session_id:
            search_kwargs["run_id"] = session_id

        results = self.memory.search(**search_kwargs)

        memory_items = []
        for result in results.get("results", []):
            memory_item = MemoryItem(
                user_id=user_id or "default",
                memory=result.get("memory", ""),
                metadata=result.get("metadata", {}),
                tags=[]
            )
            memory_items.append(memory_item)

        return memory_items

    async def remove_items(self, **kwargs) -> None:
        """Remove memory items."""
        user_id = kwargs.get("user_id")
        if user_id:
            self.memory.delete_all(user_id=user_id)
        else:
            self.memory.reset()


# Registration function for NAT
async def local_mem0_memory_client(config: LocalMem0Config, builder):
    """Create local Mem0 memory editor."""
    memory_editor = LocalMem0Editor(config)
    yield memory_editor
