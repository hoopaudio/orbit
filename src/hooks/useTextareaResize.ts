import { useState } from 'react';

export const useTextareaResize = () => {
    const [query, setQuery] = useState("");

    const handleTextareaInput = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
        const textarea = e.target;
        setQuery(textarea.value);

        // Only adjust height if we have a real textarea element with style property
        if (textarea && textarea.style) {
            textarea.style.height = 'auto';
            textarea.style.height = Math.min(textarea.scrollHeight, 200) + 'px';
        }
    };

    return {
        query,
        setQuery,
        handleTextareaInput
    };
};