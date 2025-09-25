import React, { useRef, useEffect } from 'react';
import ReactMarkdown from 'react-markdown';
import { LoadingAnimation } from './LoadingAnimation';
import { Message } from '../hooks/useStreamingQuery';
import './ResponseDisplay.scss';

interface ResponseDisplayProps {
    history: Message[];
    isLoading: boolean;
    isProducerMode?: boolean;
}

export const ResponseDisplay: React.FC<ResponseDisplayProps> = ({ history, isLoading, isProducerMode = false }) => {
    const scrollRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (scrollRef.current) {
            scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
        }
    }, [history, isLoading]);

    // In standard mode, return null if no history (original behavior)
    if (!isProducerMode && history.length === 0 && !isLoading) return null;

    return (
        <div className="response-container" ref={scrollRef} data-tauri-drag-region="false">
            <div className="response-content">
                {isProducerMode && history.length === 0 && !isLoading ? (
                    <div className="empty-state">
                        <p>No conversation yet. Start by asking a question!</p>
                    </div>
                ) : (
                    <>
                        {history.map((message, index) => (
                            <div key={index} className={`message-bubble ${message.speaker}`}>
                                <ReactMarkdown>{message.text}</ReactMarkdown>
                            </div>
                        ))}
                        {isLoading && (
                            <div className="message-bubble ai">
                                <LoadingAnimation />
                            </div>
                        )}
                    </>
                )}
            </div>
        </div>
    );
};