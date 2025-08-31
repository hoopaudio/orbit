import React, {useRef, useEffect} from 'react';
import ReactMarkdown from 'react-markdown';
import { LoadingAnimation } from './LoadingAnimation';
import { Message } from '../hooks/useStreamingQuery';
import './ResponseDisplay.scss';

interface ResponseDisplayProps {
    history: Message[];
    isLoading: boolean;
}

export const ResponseDisplay: React.FC<ResponseDisplayProps> = ({history, isLoading}) => {
    const scrollRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (scrollRef.current) {
            scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
        }
    }, [history, isLoading]);

    if (history.length === 0 && !isLoading) return null;

    return (
        <div className="response-container" ref={scrollRef} data-tauri-drag-region="false">
            <div className="response-content">
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
            </div>
        </div>
    );
};