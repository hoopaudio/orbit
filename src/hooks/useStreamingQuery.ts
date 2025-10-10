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

    const processQuery = async (query: string, selectedTracks?: any[]) => {
        if (!query.trim()) return;

        setIsLoading(true);
        const newUserMessage: Message = { speaker: 'user', text: query };
        setHistory(prev => [...prev, newUserMessage, { speaker: 'ai', text: '' }]);

        // Set up event listeners for streaming
        const unlistenChunk = await listen<string>('stream_chunk', (event) => {
            setHistory(prev => {
                const lastMessage = prev[prev.length - 1];
                const updatedLastMessage = { ...lastMessage, text: lastMessage.text + event.payload };
                return [...prev.slice(0, -1), updatedLastMessage];
            });
        });

        const unlistenDone = await listen('stream_done', () => {
            setIsLoading(false);
            // Clean up listeners
            unlistenChunk();
            unlistenDone();
        });

        try {
            // Prepare track context if provided
            const trackContext = selectedTracks && selectedTracks.length > 0
                ? JSON.stringify(selectedTracks.map(track => ({
                    index: track.index,
                    name: track.name,
                    mute: track.mute,
                    solo: track.solo,
                    arm: track.arm
                })))
                : undefined;

            // Use Python streaming implementation
            console.log("Calling process_query_python_stream with query:", query);
            if (trackContext) {
                console.log("Including track context:", trackContext);
            }
            await invoke("process_query_python_stream", {
                query,
                selected_tracks: trackContext
            });
        } catch (error) {
            console.error("Error calling process_query_python_stream:", error);
            setHistory(prev => {
                const lastMessage = prev[prev.length - 1];
                const updatedLastMessage = { ...lastMessage, text: String(error) };
                return [...prev.slice(0, -1), updatedLastMessage];
            });
            setIsLoading(false);
            // Clean up listeners on error
            unlistenChunk();
            unlistenDone();
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