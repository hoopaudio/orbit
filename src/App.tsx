import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.scss";

function App() {
  const [query, setQuery] = useState("");
  const [response, setResponse] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const setupListeners = async () => {
      // Focus input on initial mount
      if (inputRef.current) {
        inputRef.current.focus();
      }

      // Listen for window focus events to clear old state
      const unlisten = await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
        if (focused) {
          setQuery("");
          setResponse("");
          setIsLoading(false);
          // Re-focus input when window gains focus
          if (inputRef.current) {
            inputRef.current.focus();
          }
        }
      });

      return unlisten;
    };

    setupListeners().then(unlisten => {
      // Cleanup function
      return () => {
        if (unlisten) {
          unlisten();
        }
      };
    });
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!query.trim()) return;

    setIsLoading(true);
    setResponse("");

    try {
      const result = await invoke<string>("process_query", { query });
      setResponse(result);
    } catch (error) {
      setResponse(`Error: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="orbit-container" data-tauri-drag-region>
      <div className="orbit-overlay">
        <form className="orbit-form" onSubmit={handleSubmit} data-tauri-drag-region>
          <div className="input-wrapper">
            {/* Bottom layer: Warm colors */}
            <div className="warm-background"></div>
            
            {/* Top layer: Glass surface */}
            <div className="glass-layer">
              <div className="glass-highlight"></div>
            </div>
            
            {/* Input field */}
            <input
              ref={inputRef}
              className="orbit-input"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Type to Orbit"
              autoFocus
              data-tauri-drag-region="false"
            />
          </div>
          
          {(response || isLoading) && (
            <div className="response-container" data-tauri-drag-region="false">
              {isLoading ? (
                <div className="loading">
                  <span className="dot">.</span>
                  <span className="dot">.</span>
                  <span className="dot">.</span>
                </div>
              ) : (
                <p className="response-text">{response}</p>
              )}
            </div>
          )}
        </form>
      </div>
    </div>
  );
}

export default App;
