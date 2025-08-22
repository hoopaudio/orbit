import React, {useRef} from "react";
import {OrbitLogo} from "./components/OrbitLogo";
import {InputField} from "./components/InputField";
import {ResponseDisplay} from "./components/ResponseDisplay";
import {useWindowResize} from "./hooks/useWindowResize";
import {useWindowEvents} from "./hooks/useWindowEvents";
import {useStreamingQuery} from "./hooks/useStreamingQuery";
import {useTextareaResize} from "./hooks/useTextareaResize";
import "./App.scss";

function App() {
    const inputRef = useRef<HTMLTextAreaElement>(null);
    const containerRef = useRef<HTMLDivElement>(null);

    const {query, handleTextareaInput, setQuery} = useTextareaResize();
    const {response, isLoading, processQuery, setResponse, setIsLoading} = useStreamingQuery();

    useWindowResize(containerRef, response, isLoading);
    useWindowEvents(inputRef, setQuery, setResponse, setIsLoading);

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        processQuery(query);
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
                        <div className="warm-background"></div>
                        <div className="glass-layer"></div>

                        <div className="input-content">
                            <OrbitLogo/>
                            <InputField
                                ref={inputRef}
                                value={query}
                                onChange={handleTextareaInput}
                                onSubmit={handleSubmit}
                            />
                        </div>
                    </div>
                </form>

                <ResponseDisplay response={response} isLoading={isLoading}/>
            </div>
        </div>
    );
}

export default App;
