import {useState} from 'react';
import {invoke} from "@tauri-apps/api/core";
import {listen} from "@tauri-apps/api/event";

export interface Message {
    speaker: 'user' | 'ai';
    text: string;
}

export const useStreamingQuery = () => {
    const [history, setHistory] = useState<Message[]>([]);
    const [isLoading, setIsLoading] = useState(false);

    const processQuery = async (query: string) => {
        if (!query.trim()) return;

        setIsLoading(true);
        const newUserMessage: Message = { speaker: 'user', text: query };
        setHistory(prev => [...prev, newUserMessage, { speaker: 'ai', text: '' }]);

        const unlistenChunk = await listen<string>("stream_chunk", (event) => {
            setHistory(prev => {
                const lastMessage = prev[prev.length - 1];
                const updatedLastMessage = { ...lastMessage, text: lastMessage.text + event.payload };
                return [...prev.slice(0, -1), updatedLastMessage];
            });
            setIsLoading(false);
        });

        const unlistenDone = await listen("stream_done", () => {
            setIsLoading(false);
            unlistenChunk();
            unlistenDone();
            unlistenError();
        });

        const unlistenError = await listen<string>("stream_error", (event) => {
            setHistory(prev => {
                const lastMessage = prev[prev.length - 1];
                const updatedLastMessage = { ...lastMessage, text: event.payload };
                return [...prev.slice(0, -1), updatedLastMessage];
            });
            setIsLoading(false);
            unlistenChunk();
            unlistenDone();
            unlistenError();
        });

        try {
            await invoke("process_query_stream", {query});
        } catch (error) {
            setHistory(prev => {
                const lastMessage = prev[prev.length - 1];
                const updatedLastMessage = { ...lastMessage, text: error + "" };
                return [...prev.slice(0, -1), updatedLastMessage];
            });
            setIsLoading(false);
            unlistenChunk();
            unlistenDone();
            unlistenError();
        }
    };

    return {
        history,
        setHistory,
        isLoading,
        setIsLoading,
        processQuery
    };
};