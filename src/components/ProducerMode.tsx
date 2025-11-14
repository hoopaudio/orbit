import React, { useState, useEffect } from 'react';
import './ProducerMode.scss';
import { OrbitLogo } from './OrbitLogo';
import { InputField } from './InputField';
import { ResponseDisplay } from './ResponseDisplay';
import { Message } from '../hooks/useStreamingQuery';
import { AutoCompleteMenu, Command } from './AutoCompleteMenu';
import { TrackSelector } from './TrackSelector';

interface Track {
    index: number;
    name: string;
    color: number;
    is_foldable: boolean;
    mute: boolean;
    solo: boolean;
    arm: boolean;
}

interface ProducerModeProps {
    inputRef: React.RefObject<HTMLTextAreaElement>;
    query: string;
    history: Message[];
    isLoading: boolean;
    handleTextareaInput: (e: React.ChangeEvent<HTMLTextAreaElement>) => void;
    handleSubmit: (e: React.FormEvent) => void;
    handleProcessQuery: (query: string) => void;
    askOrbit?: (query: string, selectedTracks?: any[]) => Promise<void>;
    commands: Command[];
}

const ChannelPill = ({ track, onRemove }: { track: Track; onRemove: (track: Track) => void }) => (
    <div className="channel-pill">
        <span>{track.name}</span>
        <button className="remove-channel-btn" onClick={() => onRemove(track)}>&times;</button>
    </div>
);

export const ProducerMode: React.FC<ProducerModeProps> = ({
    inputRef,
    query,
    history,
    isLoading,
    handleTextareaInput,
    handleSubmit: originalHandleSubmit,
    handleProcessQuery: originalHandleProcessQuery,
    askOrbit,
    commands,
}) => {
    const [suggestions, setSuggestions] = useState<Command[]>([]);
    const [isMenuVisible, setIsMenuVisible] = useState(false);
    const [selectedIndex, setSelectedIndex] = useState(-1);
    const [selectedTracks, setSelectedTracks] = useState<Track[]>([]);
    const [isTrackSelectorOpen, setIsTrackSelectorOpen] = useState(false);

    // Reset selected index when menu visibility changes or suggestions change
    useEffect(() => {
        if (!isMenuVisible) {
            setSelectedIndex(-1);
        }
    }, [isMenuVisible]);

    useEffect(() => {
        setSelectedIndex(-1);
    }, [suggestions]);

    const handleSelectCommand = (commandName: string, shouldSubmit: boolean = false) => {
        // Update the input field
        const event = {
            target: { value: commandName },
        } as React.ChangeEvent<HTMLTextAreaElement>;
        handleTextareaInput(event);
        setIsMenuVisible(false);

        if (shouldSubmit) {
            // Execute the command IMMEDIATELY, don't wait
            handleProcessQueryWithTracks(commandName);
        }
    };

    const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
        if (isMenuVisible) {
            if (e.key === 'ArrowDown') {
                e.preventDefault();
                // Start at 0 if nothing selected, otherwise move down
                setSelectedIndex(prev => prev === -1 ? 0 : (prev + 1) % suggestions.length);
            } else if (e.key === 'ArrowUp') {
                e.preventDefault();
                // Go to last item if at beginning or nothing selected
                setSelectedIndex(prev =>
                    prev <= 0 ? suggestions.length - 1 : prev - 1
                );
            } else if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                e.stopPropagation(); // Stop event from bubbling
                // If no selection but menu is visible, select first item
                // Otherwise execute the selected command
                const indexToUse = selectedIndex === -1 ? 0 : selectedIndex;
                if (suggestions.length > 0) {
                    const commandName = suggestions[indexToUse].name;
                    // Update input and execute in one go
                    handleSelectCommand(commandName, true);
                }
                return; // Exit early to prevent further processing
            } else if (e.key === 'Escape') {
                // Allow escape to close the menu
                setIsMenuVisible(false);
                setSelectedIndex(-1);
            }
        } else if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            handleSubmitWithTracks(e);
        }
    };

    const handleTracksSelected = (tracks: Track[]) => {
        setSelectedTracks(tracks);
    };

    const handleRemoveTrack = (trackToRemove: Track) => {
        setSelectedTracks(prev => prev.filter(track => track.index !== trackToRemove.index));
    };

    const handleOpenTrackSelector = () => {
        setIsTrackSelectorOpen(true);
    };

    const handleCloseTrackSelector = () => {
        setIsTrackSelectorOpen(false);
    };

    // Enhanced query processing that includes track context
    const handleProcessQueryWithTracks = (queryString: string) => {
        if (askOrbit) {
            // Use the direct askOrbit function with track context
            askOrbit(queryString, selectedTracks);
        } else {
            // Fallback to original function (for command handling like /producer, /standard)
            originalHandleProcessQuery(queryString);
        }
    };

    const handleSubmitWithTracks = (e: React.FormEvent) => {
        e.preventDefault();
        handleProcessQueryWithTracks(query);
    };

    return (
        <div className="producer-mode-container">
            <div className="producer-mode-content">
                <div className="conversation-section">
                    <ResponseDisplay history={history} isLoading={isLoading} isProducerMode={true} />
                </div>
                <div className="context-section">
                    <button className="add-channel-btn" onClick={handleOpenTrackSelector}>@</button>
                    <div className="channels-list">
                        {selectedTracks.map(track =>
                            <ChannelPill key={track.index} track={track} onRemove={handleRemoveTrack} />
                        )}
                    </div>
                </div>
                <div className="producer-prompt-section">
                    <form className="producer-form" onSubmit={handleSubmitWithTracks}>
                        <div className="input-wrapper">
                            <div className="warm-background"></div>
                            <div className="glass-layer"></div>
                            <div className="input-content">
                                <OrbitLogo />
                                <InputField
                                    ref={inputRef}
                                    value={query}
                                    onChange={handleTextareaInput}
                                    commands={commands}
                                    onSuggestionsChange={setSuggestions}
                                    onMenuVisibilityChange={setIsMenuVisible}
                                    onKeyDown={handleKeyDown}
                                />
                            </div>
                        </div>
                    </form>
                    {isMenuVisible && <AutoCompleteMenu commands={suggestions} onSelect={handleSelectCommand} selectedIndex={selectedIndex} position="top" />}
                </div>
            </div>
            {isTrackSelectorOpen && (
                <>
                    <div className="track-selector-overlay" onClick={handleCloseTrackSelector}></div>
                    <TrackSelector
                        onTracksSelected={handleTracksSelected}
                        selectedTracks={selectedTracks}
                        onClose={handleCloseTrackSelector}
                    />
                </>
            )}
        </div>
    );
};