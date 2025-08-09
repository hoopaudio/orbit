import { useState } from 'react';

export const useTextareaResize = () => {
    const [query, setQuery] = useState("");

    const handleTextareaInput = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
        const textarea = e.target;
        setQuery(textarea.value);

        textarea.style.height = 'auto';
        textarea.style.height = Math.min(textarea.scrollHeight, 200) + 'px';
    };

    return {
        query,
        setQuery,
        handleTextareaInput
    };
};