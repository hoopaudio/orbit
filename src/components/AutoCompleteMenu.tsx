import React from 'react';
import './AutoCompleteMenu.scss';

export interface Command {
    name: string;
    description: string;
}

interface AutoCompleteMenuProps {
    commands: Command[];
    onSelect: (command: string) => void;
    selectedIndex?: number;
    position?: 'top' | 'bottom';
}

export const AutoCompleteMenu: React.FC<AutoCompleteMenuProps> = ({commands, onSelect, selectedIndex = -1, position = 'bottom'}) => {
    if (commands.length === 0) return null;

    return (
        <div className={`autocomplete-menu ${position}`}>
            <ul>
                {commands.map((command, index) => (
                    <li 
                        key={index} 
                        onClick={() => onSelect(command.name)}
                        className={selectedIndex === index ? 'selected' : ''}
                    >
                        <span className="command-name">{command.name}</span>
                        <span className="command-description">{command.description}</span>
                    </li>
                ))}
            </ul>
        </div>
    );
};
