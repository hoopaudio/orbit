import React, { forwardRef } from 'react';

interface InputFieldProps {
    value: string;
    onChange: (e: React.ChangeEvent<HTMLTextAreaElement>) => void;
    onSubmit: (e: React.FormEvent) => void;
    placeholder?: string;
}

export const InputField = forwardRef<HTMLTextAreaElement, InputFieldProps>(
    ({ value, onChange, onSubmit, placeholder = "Type to Orbit" }, ref) => {
        const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
            if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                onSubmit(e);
            }
        };

        return (
            <textarea
                ref={ref}
                value={value}
                onChange={onChange}
                placeholder={placeholder}
                className="orbit-input"
                data-tauri-drag-region="false"
                rows={1}
                onKeyDown={handleKeyDown}
            />
        );
    }
);