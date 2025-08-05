import {useState, useEffect, useRef} from "react";
import {invoke} from "@tauri-apps/api/core";
import {getCurrentWindow} from "@tauri-apps/api/window";
import {listen} from "@tauri-apps/api/event";
import ReactMarkdown from 'react-markdown';
import "./App.scss";

function App() {
    const [query, setQuery] = useState("");
    const [response, setResponse] = useState("");
    const [isLoading, setIsLoading] = useState(false);
    const inputRef = useRef<HTMLTextAreaElement>(null);
    const containerRef = useRef<HTMLDivElement>(null);

    // Resize window when response content changes
    useEffect(() => {
        if (!containerRef.current) return;
        
        const containerHeight = containerRef.current.scrollHeight;
        const windowHeight = Math.min(Math.max(containerHeight + 40, 120), 600);
        
        invoke("resize_window", {
            width: 550,
            height: windowHeight
        }).catch(console.error);
    }, [response, isLoading]);

    // Auto-resize textarea height as user types
    const handleTextareaInput = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
        const textarea = e.target;
        setQuery(textarea.value);

        // Reset height to auto to get accurate scrollHeight
        textarea.style.height = 'auto';
        textarea.style.height = Math.min(textarea.scrollHeight, 200) + 'px';
    };

    useEffect(() => {
        const setupListeners = async () => {
            // Focus input on initial mount
            if (inputRef.current) {
                inputRef.current.focus();
            }

            // Listen for window focus events to clear old state
            const unlisten = await getCurrentWindow().onFocusChanged(({payload: focused}) => {
                if (focused) {
                    setQuery("");
                    setResponse("");
                    setIsLoading(false);
                }
            });

            return unlisten;
        };

        setupListeners();
    }, []);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!query.trim()) return;

        setIsLoading(true);
        setResponse("");

        // Set up streaming event listeners
        const unlistenChunk = await listen<string>("stream_chunk", (event) => {
            setResponse(prev => prev + event.payload);
            setIsLoading(false); // Stop loading indicator on first chunk
        });

        const unlistenDone = await listen("stream_done", () => {
            setIsLoading(false);
            // Clean up listeners
            unlistenChunk();
            unlistenDone();
            unlistenError();
        });

        const unlistenError = await listen<string>("stream_error", (event) => {
            setResponse("Error processing query: " + event.payload);
            setIsLoading(false);
            // Clean up listeners
            unlistenChunk();
            unlistenDone();
            unlistenError();
        });

        try {
            await invoke("process_query_stream", {query});
        } catch (error) {
            setResponse("Error processing query: " + error);
            setIsLoading(false);
            // Clean up listeners
            unlistenChunk();
            unlistenDone();
            unlistenError();
        }
    };

    return (
        <div
            className="orbit-container"
            data-tauri-drag-region
            ref={containerRef}
            tabIndex={0}
        >
            <div className="orbit-overlay">
                <form className="orbit-form" onSubmit={handleSubmit} data-tauri-drag-region>
                    <div className="input-wrapper">
                        {/* Bottom layer: Warm colors */}
                        <div className="warm-background"></div>

                        {/* Top layer: Glass effect */}
                        <div className="glass-layer"></div>

                        {/* Logo and Input field */}
                        <div className="input-content">
                            <svg className="orbit-logo" viewBox="0 0 240 240" fill="none">
                                <defs>
                                    <linearGradient id="wireGradient" x1="0%" y1="0%" x2="100%" y2="100%">
                                        <stop offset="0%" stopColor="#ff4800"/>
                                        <stop offset="25%" stopColor="#ff6b2b"/>
                                        <stop offset="50%" stopColor="#ff8c42"/>
                                        <stop offset="75%" stopColor="#ffad5a"/>
                                        <stop offset="100%" stopColor="#ffc971"/>
                                    </linearGradient>
                                </defs>

                                <g transform="translate(120, 120)">
                                    {/* Longitude lines */}
                                    <ellipse cx="0" cy="0" rx="0" ry="100" stroke="url(#wireGradient)" strokeWidth="1"
                                             fill="none" opacity="0.9"/>
                                    <ellipse cx="0" cy="0" rx="25" ry="100" stroke="url(#wireGradient)" strokeWidth="1"
                                             fill="none" opacity="0.8"/>
                                    <ellipse cx="0" cy="0" rx="50" ry="100" stroke="url(#wireGradient)" strokeWidth="1"
                                             fill="none" opacity="0.7"/>
                                    <ellipse cx="0" cy="0" rx="70" ry="100" stroke="url(#wireGradient)" strokeWidth="1"
                                             fill="none" opacity="0.6"/>
                                    <ellipse cx="0" cy="0" rx="85" ry="100" stroke="url(#wireGradient)" strokeWidth="1"
                                             fill="none" opacity="0.5"/>
                                    <ellipse cx="0" cy="0" rx="100" ry="100" stroke="url(#wireGradient)" strokeWidth="1"
                                             fill="none" opacity="0.8"/>

                                    {/* Rotated longitude lines */}
                                    <g transform="rotate(30)">
                                        <ellipse cx="0" cy="0" rx="25" ry="100" stroke="url(#wireGradient)"
                                                 strokeWidth="1" fill="none" opacity="0.7"/>
                                        <ellipse cx="0" cy="0" rx="50" ry="100" stroke="url(#wireGradient)"
                                                 strokeWidth="1" fill="none" opacity="0.6"/>
                                        <ellipse cx="0" cy="0" rx="70" ry="100" stroke="url(#wireGradient)"
                                                 strokeWidth="1" fill="none" opacity="0.5"/>
                                        <ellipse cx="0" cy="0" rx="85" ry="100" stroke="url(#wireGradient)"
                                                 strokeWidth="1" fill="none" opacity="0.4"/>
                                    </g>

                                    <g transform="rotate(60)">
                                        <ellipse cx="0" cy="0" rx="25" ry="100" stroke="url(#wireGradient)"
                                                 strokeWidth="1" fill="none" opacity="0.7"/>
                                        <ellipse cx="0" cy="0" rx="50" ry="100" stroke="url(#wireGradient)"
                                                 strokeWidth="1" fill="none" opacity="0.6"/>
                                        <ellipse cx="0" cy="0" rx="70" ry="100" stroke="url(#wireGradient)"
                                                 strokeWidth="1" fill="none" opacity="0.5"/>
                                        <ellipse cx="0" cy="0" rx="85" ry="100" stroke="url(#wireGradient)"
                                                 strokeWidth="1" fill="none" opacity="0.4"/>
                                    </g>

                                    <g transform="rotate(90)">
                                        <ellipse cx="0" cy="0" rx="25" ry="100" stroke="url(#wireGradient)"
                                                 strokeWidth="1" fill="none" opacity="0.7"/>
                                        <ellipse cx="0" cy="0" rx="50" ry="100" stroke="url(#wireGradient)"
                                                 strokeWidth="1" fill="none" opacity="0.6"/>
                                        <ellipse cx="0" cy="0" rx="70" ry="100" stroke="url(#wireGradient)"
                                                 strokeWidth="1" fill="none" opacity="0.5"/>
                                        <ellipse cx="0" cy="0" rx="85" ry="100" stroke="url(#wireGradient)"
                                                 strokeWidth="1" fill="none" opacity="0.4"/>
                                    </g>

                                    <g transform="rotate(120)">
                                        <ellipse cx="0" cy="0" rx="25" ry="100" stroke="url(#wireGradient)"
                                                 strokeWidth="1" fill="none" opacity="0.7"/>
                                        <ellipse cx="0" cy="0" rx="50" ry="100" stroke="url(#wireGradient)"
                                                 strokeWidth="1" fill="none" opacity="0.6"/>
                                        <ellipse cx="0" cy="0" rx="70" ry="100" stroke="url(#wireGradient)"
                                                 strokeWidth="1" fill="none" opacity="0.5"/>
                                        <ellipse cx="0" cy="0" rx="85" ry="100" stroke="url(#wireGradient)"
                                                 strokeWidth="1" fill="none" opacity="0.4"/>
                                    </g>

                                    <g transform="rotate(150)">
                                        <ellipse cx="0" cy="0" rx="25" ry="100" stroke="url(#wireGradient)"
                                                 strokeWidth="1" fill="none" opacity="0.7"/>
                                        <ellipse cx="0" cy="0" rx="50" ry="100" stroke="url(#wireGradient)"
                                                 strokeWidth="1" fill="none" opacity="0.6"/>
                                        <ellipse cx="0" cy="0" rx="70" ry="100" stroke="url(#wireGradient)"
                                                 strokeWidth="1" fill="none" opacity="0.5"/>
                                        <ellipse cx="0" cy="0" rx="85" ry="100" stroke="url(#wireGradient)"
                                                 strokeWidth="1" fill="none" opacity="0.4"/>
                                    </g>

                                    {/* Latitude lines */}
                                    <ellipse cx="0" cy="0" rx="100" ry="12" stroke="url(#wireGradient)" strokeWidth="1"
                                             fill="none" opacity="0.9"/>
                                    <ellipse cx="0" cy="-25" rx="90" ry="10" stroke="url(#wireGradient)" strokeWidth="1"
                                             fill="none" opacity="0.8"/>
                                    <ellipse cx="0" cy="-50" rx="70" ry="8" stroke="url(#wireGradient)" strokeWidth="1"
                                             fill="none" opacity="0.7"/>
                                    <ellipse cx="0" cy="-75" rx="40" ry="5" stroke="url(#wireGradient)" strokeWidth="1"
                                             fill="none" opacity="0.6"/>
                                    <ellipse cx="0" cy="-90" rx="15" ry="2" stroke="url(#wireGradient)" strokeWidth="1"
                                             fill="none" opacity="0.5"/>
                                    <ellipse cx="0" cy="25" rx="90" ry="10" stroke="url(#wireGradient)" strokeWidth="1"
                                             fill="none" opacity="0.8"/>
                                    <ellipse cx="0" cy="50" rx="70" ry="8" stroke="url(#wireGradient)" strokeWidth="1"
                                             fill="none" opacity="0.7"/>
                                    <ellipse cx="0" cy="75" rx="40" ry="5" stroke="url(#wireGradient)" strokeWidth="1"
                                             fill="none" opacity="0.6"/>
                                    <ellipse cx="0" cy="90" rx="15" ry="2" stroke="url(#wireGradient)" strokeWidth="1"
                                             fill="none" opacity="0.5"/>

                                    {/* Poles */}
                                    <circle cx="0" cy="-100" r="2" fill="url(#wireGradient)" opacity="0.8"/>
                                    <circle cx="0" cy="100" r="2" fill="url(#wireGradient)" opacity="0.8"/>
                                </g>
                            </svg>
                            <textarea
                                ref={inputRef}
                                value={query}
                                onChange={handleTextareaInput}
                                placeholder="Type to Orbit"
                                className="orbit-input"
                                data-tauri-drag-region="false"
                                rows={1}
                                onKeyDown={(e) => {
                                    if (e.key === 'Enter' && !e.shiftKey) {
                                        e.preventDefault();
                                        handleSubmit(e);
                                    }
                                }}
                            />
                        </div>

                    </div>
                </form>

                {(response || isLoading) && (
                    <div className="response-container" data-tauri-drag-region="false">
                        <div className="response-content">
                            {isLoading ? (
                                <div className="loading">Processing...</div>
                            ) : (
                                <div className="response">
                                    <ReactMarkdown>{response}</ReactMarkdown>
                                </div>
                            )}
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}

export default App;
