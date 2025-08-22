import {useState} from 'react';
import {invoke} from "@tauri-apps/api/core";
import {listen} from "@tauri-apps/api/event";

export const useStreamingQuery = () => {
    const [response, setResponse] = useState("");
    const [isLoading, setIsLoading] = useState(false);

    const processQuery = async (query: string) => {
        if (!query.trim()) return;

        setIsLoading(true);
        setResponse("");

        const unlistenChunk = await listen<string>("stream_chunk", (event) => {
            setResponse(prev => prev + event.payload);
            setIsLoading(false);
        });

        const unlistenDone = await listen("stream_done", () => {
            setIsLoading(false);
            unlistenChunk();
            unlistenDone();
            unlistenError();
        });

        const unlistenError = await listen<string>("stream_error", (event) => {
            setResponse(event.payload);
            setIsLoading(false);
            unlistenChunk();
            unlistenDone();
            unlistenError();
        });

        try {
            await invoke("process_query_stream", {query});
        } catch (error) {
            setResponse(error + "");
            setIsLoading(false);
            unlistenChunk();
            unlistenDone();
            unlistenError();
        }
    };

    return {
        response,
        setResponse,
        isLoading,
        setIsLoading,
        processQuery
    };
};