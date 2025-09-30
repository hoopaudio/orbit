"""
LangChain-based bot implementation with multi-provider support.
Supports Vertex AI, OpenAI, and other providers through LangChain.
"""

import os
import asyncio
from typing import AsyncGenerator, Optional, List, Dict, Any
from langchain_core.messages import HumanMessage, SystemMessage, AIMessage
from langchain_core.language_models import BaseChatModel
from langchain.memory import ConversationBufferMemory
from langchain.schema import BaseMessage


class LangChainBot:
    """LangChain bot with support for multiple providers"""

    def __init__(self, api_key: Optional[str] = None):
        """Initialize the bot with the configured provider"""
        self.api_key = api_key
        self.provider = os.getenv("ORBIT_AI_PROVIDER", "vertex_ai").lower()
        self.llm = None
        self.memory = ConversationBufferMemory(return_messages=True)
        self.conversation_history: List[BaseMessage] = []

        # Initialize the LLM based on provider
        self._initialize_llm()

    def _initialize_llm(self):
        """Initialize the language model based on the configured provider"""

        if self.provider in ["vertex_ai", "vertexai", "vertex"]:
            try:
                from langchain_google_vertexai import ChatVertexAI

                # Get configuration from environment
                project_id = os.getenv("VERTEX_PROJECT_ID", "hoop-audio-472016")
                location = os.getenv("VERTEX_LOCATION", "us-central1")
                model_name = os.getenv("VERTEX_MODEL", "gemini-2.0-flash")

                self.llm = ChatVertexAI(
                    model_name=model_name,
                    project=project_id,
                    location=location,
                    temperature=0.7,
                    max_output_tokens=2048,
                    top_p=0.8,
                    top_k=40,
                )
                print(f"Orbit Pro Agent initialized with Vertex AI ({model_name})")
            except Exception as e:
                print(f"Failed to initialize Vertex AI: {e}")
                self.provider = "dummy"

        elif self.provider == "openai":
            try:
                from langchain_openai import ChatOpenAI

                api_key = os.getenv("OPENAI_API_KEY") or self.api_key
                model_name = os.getenv("OPENAI_MODEL", "gpt-4-turbo-preview")

                self.llm = ChatOpenAI(
                    api_key=api_key,
                    model_name=model_name,
                    temperature=0.7,
                )
                print(f"Initialized LangChain with OpenAI ({model_name})")
            except Exception as e:
                print(f"Failed to initialize OpenAI: {e}")
                self.provider = "dummy"

        elif self.provider == "openrouter":
            try:
                from langchain_openai import ChatOpenAI

                api_key = os.getenv("OPENROUTER_API_KEY") or self.api_key
                model_name = os.getenv("OPENROUTER_MODEL", "google/gemini-flash-1.5")

                self.llm = ChatOpenAI(
                    api_key=api_key,
                    base_url="https://openrouter.ai/api/v1",
                    model_name=model_name,
                    temperature=0.7,
                    default_headers={
                        "HTTP-Referer": "https://orbit-ai.app",
                        "X-Title": "Orbit AI"
                    }
                )
                print(f"Initialized LangChain with OpenRouter ({model_name})")
            except Exception as e:
                print(f"Failed to initialize OpenRouter: {e}")
                self.provider = "dummy"

        if self.provider == "dummy" or self.llm is None:
            print("Using dummy responses (no LLM configured)")

    async def ask(self, question: str, system_prompt: Optional[str] = None) -> str:
        """Send a question to the LLM and get a response"""

        # Use LangChain if available
        if self.llm is not None:
            try:
                messages = []

                # Add system prompt if provided
                if system_prompt:
                    messages.append(SystemMessage(content=system_prompt))

                # Add conversation history (last 10 messages for context)
                messages.extend(self.conversation_history[-10:])

                # Add the user's question
                messages.append(HumanMessage(content=question))

                # Get response from LLM
                response = await self.llm.ainvoke(messages)

                # Store in conversation history
                self.conversation_history.append(HumanMessage(content=question))
                self.conversation_history.append(response)

                return response.content
            except Exception as e:
                print(f"Error calling LangChain: {e}")
                return f"Error: {str(e)}"

        # Fallback to dummy responses
        return await self._dummy_response(question)

    async def stream(self, question: str, system_prompt: Optional[str] = None) -> AsyncGenerator[str, None]:
        """Stream a response from the LLM"""

        # Use LangChain streaming if available
        if self.llm is not None:
            try:
                messages = []

                if system_prompt:
                    messages.append(SystemMessage(content=system_prompt))

                # Add conversation history
                messages.extend(self.conversation_history[-10:])
                messages.append(HumanMessage(content=question))

                # Stream response
                full_response = ""
                async for chunk in self.llm.astream(messages):
                    if hasattr(chunk, 'content') and chunk.content:
                        yield chunk.content
                        full_response += chunk.content

                # Store in history
                self.conversation_history.append(HumanMessage(content=question))
                self.conversation_history.append(AIMessage(content=full_response))

                return
            except Exception as e:
                print(f"Error streaming from LangChain: {e}")
                yield f"Error: {str(e)}"
                return

        # Fallback to dummy streaming
        response = await self._dummy_response(question)
        for word in response.split():
            yield word + " "
            await asyncio.sleep(0.05)

    async def ask_with_image(self, question: str, image_path: str) -> str:
        """Handle multimodal input with image"""

        if self.llm is not None and self.provider in ["vertex_ai", "vertexai", "vertex"]:
            try:
                from langchain_core.messages import HumanMessage
                import base64

                # Read and encode image
                with open(image_path, "rb") as f:
                    image_data = base64.b64encode(f.read()).decode()

                # Create multimodal message
                message = HumanMessage(
                    content=[
                        {"type": "text", "text": question},
                        {
                            "type": "image_url",
                            "image_url": {
                                "url": f"data:image/jpeg;base64,{image_data}",
                            },
                        },
                    ]
                )

                response = await self.llm.ainvoke([message])
                return response.content
            except Exception as e:
                print(f"Error with multimodal input: {e}")
                return f"Error processing image: {str(e)}"

        # Fallback
        return f"Image analysis not available for {self.provider} provider"

    def clear_memory(self):
        """Clear conversation history"""
        self.conversation_history = []
        self.memory.clear()
        print(f"LangChain conversation memory cleared")
        return "Memory cleared"

    async def _dummy_response(self, question: str) -> str:
        """Generate dummy responses for testing"""
        await asyncio.sleep(0.1)  # Simulate processing

        if "hello" in question.lower():
            return "Hello! I'm the LangChain bot running in dummy mode. Configure a provider to get real responses."
        elif "orbit" in question.lower():
            return "Orbit is an AI-powered productivity tool using LangChain."
        elif "test" in question.lower():
            return f"Test successful! Provider: {self.provider}"
        else:
            return f"Dummy response to: '{question}'. Set ORBIT_AI_PROVIDER to use a real LLM."


