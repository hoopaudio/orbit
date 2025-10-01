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
    const isUserScrollingRef = useRef(false);
    const lastScrollHeightRef = useRef(0);
    const shouldAutoScrollRef = useRef(true);

    // Auto-scroll to bottom when new messages arrive, but only if user hasn't scrolled up
    useEffect(() => {
        if (!scrollRef.current) return;

        const container = scrollRef.current;
        const isAtBottom = container.scrollHeight - container.scrollTop - container.clientHeight < 50;

        // Determine if we should auto-scroll
        if (!isUserScrollingRef.current && (isAtBottom || shouldAutoScrollRef.current)) {
            // Scroll instantly during streaming for better readability
            container.scrollTop = container.scrollHeight;
        }

        lastScrollHeightRef.current = container.scrollHeight;
    }, [history, isLoading]);

    // Track when user manually scrolls
    useEffect(() => {
        const container = scrollRef.current;
        if (!container) return;

        let scrollTimer: NodeJS.Timeout;

        const handleScroll = () => {
            const isAtBottom = container.scrollHeight - container.scrollTop - container.clientHeight < 50;

            // Mark as user scrolling if not at bottom
            if (!isAtBottom) {
                isUserScrollingRef.current = true;
                shouldAutoScrollRef.current = false;
            }

            // Clear existing timer
            clearTimeout(scrollTimer);

            // Reset user scrolling flag after scrolling stops
            scrollTimer = setTimeout(() => {
                if (isAtBottom) {
                    isUserScrollingRef.current = false;
                    shouldAutoScrollRef.current = true;
                }
            }, 150);
        };

        container.addEventListener('scroll', handleScroll, { passive: true });
        return () => {
            container.removeEventListener('scroll', handleScroll);
            clearTimeout(scrollTimer);
        };
    }, []);

    // Reset auto-scroll when new message starts
    useEffect(() => {
        if (history.length > 0) {
            const lastMessage = history[history.length - 1];
            if (lastMessage.speaker === 'user') {
                shouldAutoScrollRef.current = true;
                isUserScrollingRef.current = false;
            }
        }
    }, [history.length]);

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
                        {history.map((message, index) => {
                            // Don't render empty AI messages during loading
                            if (message.speaker === 'ai' && message.text === '' && isLoading && index === history.length - 1) {
                                return (
                                    <div key={index} className="message-bubble ai loading">
                                        <LoadingAnimation />
                                    </div>
                                );
                            }
                            return (
                                <div key={index} className={`message-bubble ${message.speaker}`}>
                                    <ReactMarkdown>{message.text}</ReactMarkdown>
                                </div>
                            );
                        })}
                    </>
                )}
            </div>
        </div>
    );
};