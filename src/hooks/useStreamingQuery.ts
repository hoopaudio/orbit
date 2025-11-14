// TODO: Refactor to avoid double storage of conversation history
// The server maintains conversation history, but we also store it here for UI display
// Consider fetching history from server or finding a better pattern

import {useState} from 'react';
import {invoke} from "@tauri-apps/api/core";

export interface Message {
    speaker: 'user' | 'ai';
    text: string;
}

export const useStreamingQuery = () => {
    const [history, setHistory] = useState<Message[]>([]);
    const [isLoading, setIsLoading] = useState(false);

    const askOrbit = async (query: string, selectedTracks?: any[]) => {
        if (!query.trim()) return;

        setIsLoading(true);
        const newUserMessage: Message = { speaker: 'user', text: query };
        setHistory(prev => [...prev, newUserMessage]);

        try {
            // Build the query with track context if provided
            let fullQuery = query;
            if (selectedTracks && selectedTracks.length > 0) {
                const trackInfo = selectedTracks.map(track =>
                    `Track ${track.index}: ${track.name} (mute: ${track.mute}, solo: ${track.solo}, arm: ${track.arm})`
                ).join('\n');
                fullQuery = `Context - Selected tracks:\n${trackInfo}\n\nQuery: ${query}`;
            }

            // Call the ask_orbit command
            console.log("Calling ask_orbit with query:", fullQuery);
            const response = await invoke<string>("ask_orbit", {
                message: fullQuery
            });

            // Add the AI response to history (keeping for UI display)
            // TODO: Refactor to avoid double storage with server-side history
            const aiMessage: Message = { speaker: 'ai', text: response };
            setHistory(prev => [...prev, aiMessage]);
        } catch (error) {
            console.error("Error calling ask_orbit:", error);
            const errorMessage: Message = {
                speaker: 'ai',
                text: `Error: ${error}`
            };
            setHistory(prev => [...prev, errorMessage]);
        } finally {
            setIsLoading(false);
        }
    };

    return {
        history,
        setHistory,
        isLoading,
        setIsLoading,
        askOrbit
    };
};