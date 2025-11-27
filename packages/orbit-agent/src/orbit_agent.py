"""
Orbit Agent - React Agent pattern for controlling Ableton Live.
Provides a LangGraph-based agent using React (Reason + Act) pattern.
"""

from __future__ import annotations

import logging
from dataclasses import dataclass, field
from datetime import UTC, datetime
from typing import Dict, List, Literal, Sequence, cast

from langchain.chat_models import init_chat_model
from langchain_core.language_models import BaseChatModel
from langchain_core.messages import AIMessage, AnyMessage
from langchain_core.tools import BaseTool
from langgraph.checkpoint.memory import MemorySaver
from langgraph.graph import StateGraph, add_messages
from langgraph.managed import IsLastStep
from langgraph.prebuilt import ToolNode
from langgraph.runtime import Runtime
from typing_extensions import Annotated

from .context import OrbitContext
from .prompt import ORBIT_SYSTEM_PROMPT

logger = logging.getLogger(__name__)


@dataclass
class OrbitInputState:
    """Input state for the Orbit Agent."""

    messages: Annotated[Sequence[AnyMessage], add_messages] = field(  # type: ignore
        default_factory=list
    )
    """Messages tracking the agent's conversation state."""


@dataclass
class OrbitState(OrbitInputState):
    """Complete state for the Orbit Agent."""

    is_last_step: IsLastStep = field(default=False)
    """Indicates if this is the last step before hitting recursion limit."""


def load_chat_model(fully_specified_name: str) -> BaseChatModel:
    """Load a chat model from a fully specified name."""
    provider, model = fully_specified_name.split("/", maxsplit=1)
    return init_chat_model(model, model_provider=provider)


async def call_model(
    state: OrbitState, runtime: Runtime[OrbitContext]
) -> Dict[str, List[AIMessage]]:
    """Call the LLM for the Orbit Agent.

    This function prepares the system prompt with current time,
    initializes the model with tools, and processes the response.
    """
    # Initialize the model with tool binding
    from .tools import ABLETON_TOOLS

    model = load_chat_model(runtime.context.model)
    if ABLETON_TOOLS:
        model = model.bind_tools(ABLETON_TOOLS)

    # Format the system prompt with current time
    system_message = runtime.context.system_prompt.format(
        system_time=datetime.now(tz=UTC).isoformat()
    )

    if runtime.context.detailed_logs:
        logger.debug(f"System message: {system_message}")
        logger.debug(f"Input messages: {state.messages}")

    # Get the model's response
    response = cast(
        AIMessage,
        await model.ainvoke(
            [{"role": "system", "content": system_message}, *state.messages]
        ),
    )

    # Handle the case when it's the last step and model still wants to use tools
    if state.is_last_step and response.tool_calls:
        return {
            "messages": [
                AIMessage(
                    id=response.id,
                    content="Sorry, I could not complete the request in the specified number of steps.",
                )
            ]
        }

    if runtime.context.detailed_logs:
        logger.debug(f"Model response: {response}")

    # Return the model's response
    return {"messages": [response]}


def route_model_output(state: OrbitState) -> Literal["__end__", "tools"]:
    """Determine the next node based on the model's output."""
    last_message = state.messages[-1]
    if not isinstance(last_message, AIMessage):
        raise ValueError(
            f"Expected AIMessage in output edges, but got {type(last_message).__name__}"
        )

    # If there are no tool calls, we're done
    if not last_message.tool_calls:
        return "__end__"

    # Otherwise execute the requested tools
    return "tools"


class OrbitAgent:
    """React Agent for Orbit music production control."""

    def __init__(
        self,
        context: OrbitContext | None = None,
    ):
        """Initialize the Orbit React Agent.

        Args:
            context: Configuration context for the agent
        """
        self.context = context or OrbitContext()
        self.graph = self._build_graph()

        # Import tools to get count for logging
        from .tools import ABLETON_TOOLS

        logger.info(f"OrbitAgent initialized with {len(ABLETON_TOOLS)} tools")

    def _build_graph(self):
        """Build and compile the React Agent graph."""
        try:
            logger.debug("Building OrbitAgent React graph")

            # Import tools directly
            from .tools import ABLETON_TOOLS

            # Create the state graph with context
            builder = StateGraph(
                OrbitState, input_schema=OrbitInputState, context_schema=OrbitContext
            )

            # Add nodes
            builder.add_node(call_model)
            builder.add_node("tools", ToolNode(ABLETON_TOOLS))

            # Set the entry point
            builder.add_edge("__start__", "call_model")

            # Add conditional edges from call_model
            builder.add_conditional_edges("call_model", route_model_output)

            # Add edge from tools back to call_model
            builder.add_edge("tools", "call_model")

            # Compile the graph with memory saver for conversation continuity
            memory = MemorySaver()
            graph = builder.compile(checkpointer=memory, name="Orbit React Agent")
            logger.info("OrbitAgent React graph compiled successfully with memory")
            return graph

        except Exception as ex:
            logger.error(f"Failed to build OrbitAgent graph: {ex}")
            raise

    async def stream(self, user_input: str, thread_id: str = "default", **kwargs):
        """Stream the agent's response for a user input."""
        # Create input with new user message - LangGraph will handle state accumulation
        input_state = {"messages": [{"role": "user", "content": user_input}]}

        # Stream through the graph with thread_id for conversation continuity
        try:
            async for event in self.graph.astream(
                input_state,
                config={"configurable": {"thread_id": thread_id}},
                context=self.context,
                **kwargs,
            ):
                # Extract content from call_model responses
                for node_name, node_output in event.items():
                    if node_name == "call_model" and "messages" in node_output:
                        last_message = node_output["messages"][-1]
                        if isinstance(last_message, AIMessage) and last_message.content:
                            yield last_message.content

        except Exception as ex:
            logger.error(f"Error streaming agent response: {ex}")
            yield f"Error: {ex}"

    async def run(self, user_input: str, **kwargs) -> str:
        """Run the agent with user input and return the complete response."""
        response_chunks = []
        async for chunk in self.stream(user_input, **kwargs):
            response_chunks.append(chunk)

        return "".join(response_chunks) if response_chunks else "No response generated."
