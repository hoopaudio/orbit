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

        try {
            // Use Python implementation (non-streaming)
            console.log("Calling process_query_python with query:", query);
            const response = await invoke<string>("process_query_python", {query});
            console.log("Got response:", response);

            // Update the last AI message with the full response
            setHistory(prev => {
                const lastMessage = prev[prev.length - 1];
                const updatedLastMessage = { ...lastMessage, text: response };
                return [...prev.slice(0, -1), updatedLastMessage];
            });
        } catch (error) {
            console.error("Error calling process_query_python:", error);
            setHistory(prev => {
                const lastMessage = prev[prev.length - 1];
                const updatedLastMessage = { ...lastMessage, text: String(error) };
                return [...prev.slice(0, -1), updatedLastMessage];
            });
        } finally {
            setIsLoading(false);
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