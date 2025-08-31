import React, {forwardRef, useEffect} from 'react';
import {Command} from './AutoCompleteMenu';

interface InputFieldProps {
    value: string;
    onChange: (e: React.ChangeEvent<HTMLTextAreaElement>) => void;
    onSubmit: (e: React.FormEvent) => void;
    placeholder?: string;
    commands?: Command[];
    onSuggestionsChange: (suggestions: Command[]) => void;
    onMenuVisibilityChange: (isVisible: boolean) => void;
    onKeyDown: (e: React.KeyboardEvent<HTMLTextAreaElement>) => void;
}

export const InputField = forwardRef<HTMLTextAreaElement, InputFieldProps>(
    (
        {
            value,
            onChange,
            onSubmit,
            placeholder = "Type to Orbit",
            commands = [],
            onSuggestionsChange,
            onMenuVisibilityChange,
            onKeyDown,
        },
        ref
    ) => {
        useEffect(() => {
            if (value.startsWith('/')) {
                const query = value.substring(1).toLowerCase();
                const filteredCommands = commands.filter(command =>
                    command.name.toLowerCase().includes(query)
                );
                onSuggestionsChange(filteredCommands);
                onMenuVisibilityChange(filteredCommands.length > 0);
            } else {
                onSuggestionsChange([]);
                onMenuVisibilityChange(false);
            }
        }, [value, commands, onSuggestionsChange, onMenuVisibilityChange]);

        return (
            <div className="input-field-container">
                <textarea
                    ref={ref}
                    value={value}
                    onChange={onChange}
                    placeholder={placeholder}
                    className="orbit-input"
                    data-tauri-drag-region="false"
                    rows={1}
                    onKeyDown={onKeyDown}
                />
            </div>
        );
    }
);