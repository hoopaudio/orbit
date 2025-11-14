"""
Patch for ChatLiteLLM to fix model attribute access.
The ReAct agent expects model and model_name attributes but ChatLiteLLM returns None.
"""

try:
    from langchain_litellm import ChatLiteLLM

    # Store original __init__
    original_init = ChatLiteLLM.__init__

    def patched_init(self, *args, **kwargs):
        # Call original init
        original_init(self, *args, **kwargs)

        # Store the model from kwargs - ChatLiteLLM doesn't properly expose these
        # The ReAct agent needs these to be strings, not None
        model_value = kwargs.get('model') or kwargs.get('model_name')
        if model_value:
            self.model = model_value
            self.model_name = model_value
        else:
            # If no model specified, ensure attributes exist as empty strings
            # This prevents the TypeError in agent.py
            self.model = ""
            self.model_name = ""

    # Apply the patch
    ChatLiteLLM.__init__ = patched_init
    print("Applied ChatLiteLLM patch for model attribute access")

except ImportError as e:
    print(f"Could not apply ChatLiteLLM patch: {e}")
except Exception as e:
    print(f"Error applying ChatLiteLLM patch: {e}")