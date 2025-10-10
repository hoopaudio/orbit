import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './TrackSelector.scss';

interface Track {
    index: number;
    name: string;
    color: number;
    is_foldable: boolean;
    mute: boolean;
    solo: boolean;
    arm: boolean;
}

interface TrackSelectorProps {
    onTracksSelected: (tracks: Track[]) => void;
    selectedTracks: Track[];
    onClose: () => void;
}

export const TrackSelector: React.FC<TrackSelectorProps> = ({
    onTracksSelected,
    selectedTracks,
    onClose
}) => {
    const [tracks, setTracks] = useState<Track[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const hasInitialized = useRef(false);

    useEffect(() => {
        if (!hasInitialized.current) {
            hasInitialized.current = true;
            fetchTracks();
        }
    }, []);

    const fetchTracks = async () => {
        try {
            setLoading(true);

            // Call the Rust command to get tracks from Ableton Live
            const result = await invoke('get_ableton_tracks') as string;
            console.log('Raw track response:', result);

            // The response should now be a JSON string with track data
            let parsedTracks: Track[] = [];

            try {
                const trackData = JSON.parse(result);
                if (Array.isArray(trackData)) {
                    parsedTracks = trackData.map((track: any) => ({
                        index: track.index || 0,
                        name: track.name || `Track ${track.index}`,
                        color: track.color || 0,
                        is_foldable: track.is_foldable || false,
                        mute: track.mute || false,
                        solo: track.solo || false,
                        arm: track.arm || false,
                    }));
                } else {
                    throw new Error('Track data is not an array');
                }
            } catch (parseErr) {
                console.error('Failed to parse track JSON:', parseErr);
                // If parsing fails, it might be an error message from the backend
                if (result.includes('Failed to get tracks from Ableton') || result.includes('No tracks found')) {
                    throw new Error('Could not connect to Ableton Live. Make sure Ableton is running and the OrbitRemote script is loaded.');
                } else {
                    throw new Error('Invalid track data format');
                }
            }

            if (parsedTracks.length === 0) {
                throw new Error('No tracks found in Ableton Live. Make sure your project has tracks and the OrbitRemote script is loaded.');
            }

            setTracks(parsedTracks);
        } catch (err) {
            setError('Failed to fetch tracks from Ableton Live');
            console.error('Error fetching tracks:', err);
        } finally {
            setLoading(false);
        }
    };

    const handleTrackToggle = (track: Track) => {
        const isSelected = selectedTracks.some(t => t.index === track.index);

        if (isSelected) {
            // Remove track
            const newSelection = selectedTracks.filter(t => t.index !== track.index);
            onTracksSelected(newSelection);
        } else {
            // Add track
            const newSelection = [...selectedTracks, track];
            onTracksSelected(newSelection);
        }
    };

    const isTrackSelected = (track: Track) => {
        return selectedTracks.some(t => t.index === track.index);
    };

    if (loading) {
        return (
            <div className="track-selector">
                <div className="track-selector-header">
                    <h3>Loading tracks...</h3>
                    <button className="close-btn" onClick={onClose}>&times;</button>
                </div>
                <div className="loading-spinner">⏳</div>
            </div>
        );
    }

    if (error) {
        return (
            <div className="track-selector">
                <div className="track-selector-header">
                    <h3>Error</h3>
                    <button className="close-btn" onClick={onClose}>&times;</button>
                </div>
                <div className="error-message">
                    {error}
                    <button onClick={fetchTracks} className="retry-btn">Retry</button>
                </div>
            </div>
        );
    }

    return (
        <div className="track-selector">
            <div className="track-selector-header">
                <h3>Select Tracks</h3>
                <button className="close-btn" onClick={onClose}>&times;</button>
            </div>
            <div className="tracks-list">
                {tracks.map(track => (
                    <div
                        key={track.index}
                        className={`track-item ${isTrackSelected(track) ? 'selected' : ''}`}
                        onClick={() => handleTrackToggle(track)}
                    >
                        <div className="track-info">
                            <span className="track-index">{track.index}</span>
                            <span className="track-name">{track.name}</span>
                        </div>
                        <div className="track-status">
                            {track.mute && <span className="status-icon mute">🔇</span>}
                            {track.solo && <span className="status-icon solo">🔊</span>}
                            {track.arm && <span className="status-icon arm">🔴</span>}
                        </div>
                        <div className="track-checkbox">
                            {isTrackSelected(track) && '✓'}
                        </div>
                    </div>
                ))}
            </div>
            <div className="track-selector-footer">
                <span>{selectedTracks.length} track(s) selected</span>
                <button className="done-btn" onClick={onClose}>Done</button>
            </div>
        </div>
    );
};