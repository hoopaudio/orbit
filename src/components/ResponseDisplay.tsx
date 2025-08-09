import React from 'react';
import ReactMarkdown from 'react-markdown';

interface ResponseDisplayProps {
    response: string;
    isLoading: boolean;
}

export const ResponseDisplay: React.FC<ResponseDisplayProps> = ({ response, isLoading }) => {
    if (!response && !isLoading) return null;

    return (
        <div className="response-container" data-tauri-drag-region="false">
            <div className="response-content">
                {isLoading ? (
                    <div className="loading">Processing...</div>
                ) : (
                    <div className="response">
                        <ReactMarkdown>{response}</ReactMarkdown>
                    </div>
                )}
            </div>
        </div>
    );
};