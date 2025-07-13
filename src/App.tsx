import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.scss";

function App() {
  const [query, setQuery] = useState("");
  const [response, setResponse] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    // Focus input when component mounts
    if (inputRef.current) {
      inputRef.current.focus();
    }

    // Handle global escape key
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        hideWindow();
      }
    };

    window.addEventListener("keydown", handleEscape);
    return () => window.removeEventListener("keydown", handleEscape);
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!query.trim() || isLoading) return;

    setIsLoading(true);
    try {
      const result = await invoke("process_query", { query });
      setResponse(result as string);
      setQuery("");
    } catch (error) {
      console.error("Error processing query:", error);
      setResponse("Sorry, I couldn't process that request.");
    } finally {
      setIsLoading(false);
    }
  };

  const hideWindow = async () => {
    try {
      await invoke("hide_window");
    } catch (error) {
      console.error("Error hiding window:", error);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      hideWindow();
    }
  };

  return (
    <div className="orbit-container" data-tauri-drag-region>
      <div className="orbit-overlay" data-tauri-drag-region>
        <form onSubmit={handleSubmit} className="orbit-form" data-tauri-drag-region>
          <div className="input-wrapper" data-tauri-drag-region="false">
            <input
              ref={inputRef}
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Type to Orbit"
              className="orbit-input"
              disabled={isLoading}
              data-tauri-drag-region="false"
            />
          </div>

          {(response || isLoading) && (
            <div className="response-container" data-tauri-drag-region="false">
            <p className="response-text">
              {isLoading ? (
                <span className="loading">
                  <span className="dot">.</span>
                  <span className="dot">.</span>
                  <span className="dot">.</span>
                </span>
              ) : (
                response
              )}
            </p>
          </div>
        )}
        </form>
      </div>
    </div>
  );
}

export default App;
