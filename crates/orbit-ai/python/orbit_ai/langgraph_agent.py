"""
LangGraph-based agent implementation for Orbit AI with streaming and agentic capabilities.
"""

import os
import asyncio
from typing import AsyncGenerator, Optional, List, Dict, Any, TypedDict, Annotated
from typing_extensions import TypedDict
from langchain_core.messages import HumanMessage, SystemMessage, AIMessage, BaseMessage
from langchain_core.language_models import BaseChatModel
from langgraph.prebuilt import create_react_agent
from langgraph.config import get_stream_writer
from langchain.memory import ConversationBufferMemory
from .system_prompt import ORBIT_SYSTEM_PROMPT
from .ableton_tools import ABLETON_TOOLS


class AgentState(TypedDict):
    """State for the agent."""
    messages: List[BaseMessage]
    chat_history: List[BaseMessage]
    system_prompt: str


class LangGraphAgent:
    """LangGraph-based agent with enhanced streaming and tool capabilities"""

    def __init__(self, api_key: Optional[str] = None):
        """Initialize the agent with the configured provider"""
        self.api_key = api_key
        self.provider = os.getenv("ORBIT_AI_PROVIDER", "vertex_ai").lower()
        self.llm: Optional[BaseChatModel] = None
        self.memory = ConversationBufferMemory(return_messages=True)
        self.conversation_history: List[BaseMessage] = []
        self.agent = None

        # Initialize the LLM based on provider
        self._initialize_llm()

        # Initialize the agent with tools if LLM is available
        if self.llm is not None:
            self._initialize_agent()

    def _warmup_connection(self):
        """Warm up the connection to the LLM provider"""
        if self.llm is None:
            return

        try:
            print("Warming up LLM connection...")
            test_messages = [
                SystemMessage(content="You are a helpful assistant."),
                HumanMessage(content="Hi")
            ]

            try:
                loop = asyncio.get_event_loop()
            except RuntimeError:
                loop = asyncio.new_event_loop()
                asyncio.set_event_loop(loop)

            async def warmup():
                if self.llm is not None:
                    async for _ in self.llm.astream(test_messages):
                        break

            loop.run_until_complete(warmup())
            print("LLM connection warmed up successfully")
        except Exception as e:
            print(f"Warning: Failed to warm up connection: {e}")

    def _initialize_llm(self):
        """Initialize the language model based on the configured provider"""

        if self.provider in ["vertex_ai", "vertexai", "vertex"]:
            try:
                from langchain_google_vertexai import ChatVertexAI

                project_id = os.getenv("VERTEX_PROJECT_ID", "hoop-audio-472016")
                location = os.getenv("VERTEX_LOCATION", "global")
                model_name = os.getenv("VERTEX_MODEL", "gemini-2.5-flash-lite-preview-09-2025")

                self.llm = ChatVertexAI(
                    model=model_name,
                    project=project_id,
                    location=location,
                    temperature=0.7,
                    max_output_tokens=2048,
                    top_p=0.8,
                    top_k=40,
                )
                print(f"Orbit Pro Agent initialized with Vertex AI ({model_name})")
                self._warmup_connection()
            except Exception as e:
                print(f"Failed to initialize Vertex AI: {e}")
                self.provider = "dummy"

        elif self.provider == "openai":
            try:
                from langchain_openai import ChatOpenAI
                from pydantic import SecretStr

                api_key = os.getenv("OPENAI_API_KEY") or self.api_key
                model_name = os.getenv("OPENAI_MODEL", "gpt-4-turbo-preview")

                self.llm = ChatOpenAI(
                    api_key=SecretStr(api_key) if api_key else None,
                    model=model_name,
                    temperature=0.7,
                )
                print(f"Initialized LangGraph with OpenAI ({model_name})")
            except Exception as e:
                print(f"Failed to initialize OpenAI: {e}")
                self.provider = "dummy"

        elif self.provider == "openrouter":
            try:
                from langchain_openai import ChatOpenAI
                from pydantic import SecretStr

                api_key = os.getenv("OPENROUTER_API_KEY") or self.api_key
                model_name = os.getenv("OPENROUTER_MODEL", "google/gemini-flash-1.5")

                self.llm = ChatOpenAI(
                    api_key=SecretStr(api_key) if api_key else None,
                    base_url="https://openrouter.ai/api/v1",
                    model=model_name,
                    temperature=0.7,
                    default_headers={
                        "HTTP-Referer": "https://orbit-ai.app",
                        "X-Title": "Orbit AI"
                    }
                )
                print(f"Initialized LangGraph with OpenRouter ({model_name})")
            except Exception as e:
                print(f"Failed to initialize OpenRouter: {e}")
                self.provider = "dummy"

        if self.provider == "dummy" or self.llm is None:
            print("Using dummy responses (no LLM configured)")

    def _initialize_agent(self):
        """Initialize the LangGraph agent with Ableton tools"""
        try:
            if self.llm is None:
                print("Cannot initialize agent: LLM is not available")
                self.agent = None
                return

            print(f"Loading {len(ABLETON_TOOLS)} tools...")
            for tool in ABLETON_TOOLS:
                print(f"  - {tool.name}: {tool.description[:50]}...")

            # Create the agent using LangGraph's prebuilt create_react_agent
            self.agent = create_react_agent(
                model=self.llm,
                tools=ABLETON_TOOLS,
            )

            print(f"Successfully initialized LangGraph agent with {len(ABLETON_TOOLS)} Ableton control tools")
        except Exception as e:
            print(f"Failed to initialize agent with tools: {e}")
            import traceback
            traceback.print_exc()
            self.agent = None

    async def ask(self, question: str, system_prompt: Optional[str] = None) -> str:
        """Send a question to the agent and get a response"""

        print(f"[DEBUG] ask() called with question: {question[:50]}...")
        print(f"[DEBUG] agent exists: {self.agent is not None}")
        print(f"[DEBUG] llm exists: {self.llm is not None}")

        # Use the Orbit system prompt if not provided
        if system_prompt is None:
            system_prompt = ORBIT_SYSTEM_PROMPT

        # Use agent if available (for tool calling)
        if self.agent is not None:
            try:
                print("[DEBUG] Using LangGraph agent with tools")

                # Prepare messages with system prompt and conversation history
                messages: List[BaseMessage] = [SystemMessage(content=system_prompt)]
                messages.extend(self.conversation_history[-10:] if self.conversation_history else [])
                messages.append(HumanMessage(content=question))

                # Run the agent
                result = await self.agent.ainvoke({
                    "messages": messages
                }, config={"recursion_limit": 100})

                # Extract the final response
                final_message = result["messages"][-1]
                response_content = final_message.content if hasattr(final_message, 'content') else str(final_message)

                # Store in conversation history
                self.conversation_history.append(HumanMessage(content=question))
                self.conversation_history.append(AIMessage(content=response_content))

                return response_content

            except Exception as e:
                print(f"[ERROR] Agent execution failed: {e}")
                import traceback
                traceback.print_exc()
                # Fall back to regular LLM if agent fails
                return await self._ask_without_tools(question, system_prompt)

        # Use LangChain without tools if available
        elif self.llm is not None:
            print("[DEBUG] Using LLM without tools")
            return await self._ask_without_tools(question, system_prompt)

        # Fallback to dummy responses
        print("[DEBUG] Using dummy responses")
        return await self._dummy_response(question)

    async def _ask_without_tools(self, question: str, system_prompt: Optional[str] = None) -> str:
        """Send a question to the LLM without using tools"""
        try:
            if self.llm is None:
                return "Error: LLM is not initialized"

            if system_prompt is None:
                system_prompt = ORBIT_SYSTEM_PROMPT

            messages: List[BaseMessage] = [SystemMessage(content=system_prompt)]
            messages.extend(self.conversation_history[-10:])
            messages.append(HumanMessage(content=question))

            response = await self.llm.ainvoke(messages)

            # Store in conversation history
            self.conversation_history.append(HumanMessage(content=question))
            self.conversation_history.append(response)

            # Handle different content types
            content = response.content
            if isinstance(content, str):
                return content
            elif isinstance(content, list):
                text_parts = []
                for item in content:
                    if isinstance(item, str):
                        text_parts.append(item)
                    elif isinstance(item, dict) and 'text' in item:
                        text_parts.append(item['text'])
                return ' '.join(text_parts)
            else:
                return str(content)
        except Exception as e:
            print(f"Error calling LLM: {e}")
            return f"Error: {str(e)}"

    async def stream(self, question: str, system_prompt: Optional[str] = None) -> AsyncGenerator[str, None]:
        """Stream a response from the agent with enhanced streaming modes"""

        print(f"[DEBUG] stream() called with: {question[:50]}...")
        print(f"[DEBUG] agent exists: {self.agent is not None}")

        if system_prompt is None:
            system_prompt = ORBIT_SYSTEM_PROMPT

        # Try to use the agent first for tool calling with streaming
        if self.agent is not None:
            try:
                print("[DEBUG] Using LangGraph agent with streaming")

                # Prepare messages
                messages: List[BaseMessage] = [SystemMessage(content=system_prompt)]
                messages.extend(self.conversation_history[-10:] if self.conversation_history else [])
                messages.append(HumanMessage(content=question))

                full_response = ""
                tool_calls_shown = set()

                # Stream with updates mode for real-time feedback
                async for chunk in self.agent.astream(
                    {"messages": messages},
                    stream_mode="updates",
                    config={"recursion_limit": 100}  # Increase limit for complex mixing tasks
                ):
                    # Check for agent updates
                    if "agent" in chunk:
                        agent_messages = chunk["agent"].get("messages", [])

                        for msg in agent_messages:
                            has_tool_calls = hasattr(msg, 'tool_calls') and msg.tool_calls
                            has_content = hasattr(msg, 'content') and msg.content

                            # Handle tool calls first
                            if has_tool_calls:
                                for tool_call in msg.tool_calls:
                                    tool_name = tool_call.get('name', 'Unknown')
                                    if tool_name not in tool_calls_shown:
                                        tool_calls_shown.add(tool_name)
                                        yield f"\n\n🎛️ Using {tool_name}\n\n&nbsp;\n\n"

                            # Handle AI responses
                            if has_content:
                                content = msg.content
                                if isinstance(content, str):
                                    full_response = content
                                    yield content + "\n\n&nbsp;\n\n"
                                elif isinstance(content, list):
                                    for item in content:
                                        if isinstance(item, str):
                                            yield item
                                            full_response += item

                    # Handle tool responses
                    if "tools" in chunk:
                        tool_messages = chunk["tools"].get("messages", [])
                        for msg in tool_messages:
                            if hasattr(msg, 'content'):
                                yield f"→ {msg.content}\n\n"

                # Update conversation history
                if full_response:
                    self.conversation_history.append(HumanMessage(content=question))
                    self.conversation_history.append(AIMessage(content=full_response))

                return

            except Exception as e:
                print(f"[ERROR] Agent streaming failed: {e}")
                import traceback
                traceback.print_exc()
                yield f"\n\n**⚠️ Agent error occurred. Falling back to basic mode.**\n\n"

        # Fallback to LLM streaming without tools
        if self.llm is not None:
            try:
                print("[DEBUG] Using LLM streaming without tools")

                messages = [SystemMessage(content=system_prompt)]
                messages.extend(self.conversation_history[-10:])
                messages.append(HumanMessage(content=question))

                full_response = ""
                async for chunk in self.llm.astream(messages):
                    if hasattr(chunk, 'content') and chunk.content:
                        content = chunk.content
                        if isinstance(content, str):
                            text = content
                        elif isinstance(content, list):
                            text_parts = []
                            for item in content:
                                if isinstance(item, str):
                                    text_parts.append(item)
                                elif isinstance(item, dict) and 'text' in item:
                                    text_parts.append(item['text'])
                            text = ' '.join(text_parts)
                        else:
                            text = str(content)

                        full_response += text
                        yield text

                # Store in history
                self.conversation_history.append(HumanMessage(content=question))
                self.conversation_history.append(AIMessage(content=full_response))

                return
            except Exception as e:
                print(f"Error streaming from LLM: {e}")
                yield f"Error: {str(e)}"
                return

        # Fallback to dummy streaming
        response = await self._dummy_response(question)
        for word in response.split():
            yield word + " "
            await asyncio.sleep(0.05)

    async def stream_with_tools(self, question: str, system_prompt: Optional[str] = None) -> AsyncGenerator[Dict[str, Any], None]:
        """Advanced streaming with structured output for tool calls and responses"""

        if system_prompt is None:
            system_prompt = ORBIT_SYSTEM_PROMPT

        if self.agent is not None:
            try:
                messages: List[BaseMessage] = [SystemMessage(content=system_prompt)]
                messages.extend(self.conversation_history[-10:] if self.conversation_history else [])
                messages.append(HumanMessage(content=question))

                # Stream with all modes for maximum visibility
                async for stream_mode, chunk in self.agent.astream(
                    {"messages": messages},
                    stream_mode=["messages", "updates", "debug", "custom"]
                ):
                    yield {
                        "type": stream_mode,
                        "content": chunk,
                        "timestamp": asyncio.get_event_loop().time()
                    }

            except Exception as e:
                yield {
                    "type": "error",
                    "content": str(e),
                    "timestamp": asyncio.get_event_loop().time()
                }

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

                # Handle different content types
                content = response.content
                if isinstance(content, str):
                    return content
                elif isinstance(content, list):
                    text_parts = []
                    for item in content:
                        if isinstance(item, str):
                            text_parts.append(item)
                        elif isinstance(item, dict) and 'text' in item:
                            text_parts.append(item['text'])
                    return ' '.join(text_parts)
                else:
                    return str(content)
            except Exception as e:
                print(f"Error with multimodal input: {e}")
                return f"Error processing image: {str(e)}"

        # Fallback
        return f"Image analysis not available for {self.provider} provider"

    def clear_memory(self):
        """Clear conversation history"""
        self.conversation_history = []
        self.memory.clear()
        print(f"LangGraph agent conversation memory cleared")
        return "Memory cleared"

    async def _dummy_response(self, question: str) -> str:
        """Generate dummy responses for testing"""
        await asyncio.sleep(0.1)  # Simulate processing

        if "hello" in question.lower():
            return "Hello! I'm the LangGraph agent running in dummy mode. Configure a provider to get real responses."
        elif "orbit" in question.lower():
            return "Orbit is an AI-powered music production tool using LangGraph for enhanced streaming and agentic capabilities."
        elif "test" in question.lower():
            return f"Test successful! Provider: {self.provider}"
        else:
            return f"Dummy response to: '{question}'. Set ORBIT_AI_PROVIDER to use a real LLM."

