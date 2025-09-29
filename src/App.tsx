import React, {useRef, useState, useEffect} from "react";
import {useWindowResize} from "./hooks/useWindowResize";
import {useWindowEvents} from "./hooks/useWindowEvents";
import {useStreamingQuery} from "./hooks/useStreamingQuery";
import {useTextareaResize} from "./hooks/useTextareaResize";
import {StandardMode} from "./components/StandardMode";
import {ProducerMode} from "./components/ProducerMode";
import {Command} from "./components/AutoCompleteMenu";
import {invoke} from "@tauri-apps/api/core";
import "./App.scss";

function App() {
    const [isProducerMode, setIsProducerMode] = useState(false);
    const inputRef = useRef<HTMLTextAreaElement>(null);
    const containerRef = useRef<HTMLDivElement>(null);

    const {query, handleTextareaInput, setQuery} = useTextareaResize();
    const {history, isLoading, processQuery, setHistory, setIsLoading} = useStreamingQuery();

    const commands: Command[] = [
        {name: "/producer", description: "Toggle Producer Mode"},
        {name: "/standard", description: "Toggle Standard Mode"},
    ];

    // Only use dynamic resize in standard mode
    useWindowResize(containerRef, history.map(h => h.text).join('\n'), isLoading, isProducerMode);
    useWindowEvents(inputRef, setQuery, () => setHistory([]), setIsLoading);

    // Add keyboard shortcut to clear conversation (Cmd+K)
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.metaKey && e.key === 'k') {
                e.preventDefault();
                setHistory([]);
                setQuery("");
            }
        };

        document.addEventListener('keydown', handleKeyDown);
        return () => document.removeEventListener('keydown', handleKeyDown);
    }, []);

    // Handle producer mode window resizing
    useEffect(() => {
        const handleResize = async () => {
            if (isProducerMode) {
                try {
                    // Use the specialized command that handles all the native positioning
                    await invoke("resize_window_for_producer_mode");
                } catch (error) {
                    console.error("Failed to resize for producer mode:", error);
                    // Fallback to a reasonable default
                    invoke("resize_window", {
                        width: 550,
                        height: window.screen.availHeight - 100
                    }).catch(console.error);
                }
            } else {
                // When switching back to standard mode, resize and reposition the window
                // Small delay to ensure DOM has updated
                setTimeout(() => {
                    if (containerRef.current) {
                        const containerHeight = containerRef.current.scrollHeight;
                        const windowHeight = Math.min(Math.max(containerHeight + 60, 185), 1050);
                        invoke("resize_and_reposition_for_standard_mode", {
                            width: 550,
                            height: windowHeight
                        }).catch(console.error);
                    }
                }, 100);
            }
        };
        
        handleResize();
    }, [isProducerMode]);

    const handleProcessQuery = (queryString: string) => {
        if (queryString.trim() === "/producer") {
            setIsProducerMode(true);  // Set to true, don't toggle
            setQuery("");
            return;
        }
        if (queryString.trim() === "/standard") {
            setIsProducerMode(false);
            setQuery("");
            return;
        }
        processQuery(queryString);
    };

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        handleProcessQuery(query);
    };

    if (isProducerMode) {
        return <ProducerMode
            inputRef={inputRef}
            query={query}
            history={history}
            isLoading={isLoading}
            handleTextareaInput={handleTextareaInput}
            handleSubmit={handleSubmit}
            handleProcessQuery={handleProcessQuery}
            commands={commands}
        />;
    }

    return (
        <StandardMode
            containerRef={containerRef}
            inputRef={inputRef}
            query={query}
            history={history}
            isLoading={isLoading}
            handleTextareaInput={handleTextareaInput}
            handleSubmit={handleSubmit}
            handleProcessQuery={handleProcessQuery}
            commands={commands}
        />
    );
}

export default App;
