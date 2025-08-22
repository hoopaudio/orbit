import React from 'react';
import ReactMarkdown from 'react-markdown';
import { LoadingAnimation } from './LoadingAnimation';

interface ResponseDisplayProps {
    response: string;
    isLoading: boolean;
}

export const ResponseDisplay: React.FC<ResponseDisplayProps> = ({response, isLoading}) => {
    if (!response && !isLoading) return null;

    return (
        <div className="response-container" data-tauri-drag-region="false">
            <div className="response-content">
                {isLoading ? (
                    <LoadingAnimation />
                ) : (
                    <div className="response">
                        <ReactMarkdown>{response}</ReactMarkdown>
                    </div>
                )}
            </div>
        </div>
    );
};