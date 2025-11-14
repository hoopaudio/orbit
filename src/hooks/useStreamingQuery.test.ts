import { renderHook, act, waitFor } from '@testing-library/react';
import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest';
import { useStreamingQuery } from './useStreamingQuery';

// Mock the Tauri invoke function
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

import { invoke } from '@tauri-apps/api/core';

describe('useStreamingQuery', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('askOrbit', () => {
    it('should handle successful query without tracks', async () => {
      const mockResponse = 'This is a response from Orbit';
      (invoke as any).mockResolvedValueOnce(mockResponse);

      const { result } = renderHook(() => useStreamingQuery());

      expect(result.current.history).toEqual([]);
      expect(result.current.isLoading).toBe(false);

      await act(async () => {
        await result.current.askOrbit('What is the weather?');
      });

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(invoke).toHaveBeenCalledWith('ask_orbit', {
        message: 'What is the weather?'
      });

      expect(result.current.history).toEqual([
        { speaker: 'user', text: 'What is the weather?' },
        { speaker: 'ai', text: mockResponse }
      ]);
    });

    it('should handle successful query with selected tracks', async () => {
      const mockResponse = 'Track information processed';
      (invoke as any).mockResolvedValueOnce(mockResponse);

      const selectedTracks = [
        { index: 0, name: 'Drums', mute: false, solo: true, arm: false },
        { index: 1, name: 'Bass', mute: true, solo: false, arm: true }
      ];

      const { result } = renderHook(() => useStreamingQuery());

      await act(async () => {
        await result.current.askOrbit('Analyze these tracks', selectedTracks);
      });

      const expectedMessage = `Context - Selected tracks:
Track 0: Drums (mute: false, solo: true, arm: false)
Track 1: Bass (mute: true, solo: false, arm: true)

Query: Analyze these tracks`;

      expect(invoke).toHaveBeenCalledWith('ask_orbit', {
        message: expectedMessage
      });

      expect(result.current.history).toEqual([
        { speaker: 'user', text: 'Analyze these tracks' },
        { speaker: 'ai', text: mockResponse }
      ]);
    });

    it('should handle errors gracefully', async () => {
      const mockError = new Error('Failed to connect to Orbit');
      (invoke as any).mockRejectedValueOnce(mockError);

      const { result } = renderHook(() => useStreamingQuery());

      await act(async () => {
        await result.current.askOrbit('Test query');
      });

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.history).toEqual([
        { speaker: 'user', text: 'Test query' },
        { speaker: 'ai', text: `Error: ${mockError}` }
      ]);
    });

    it('should not process empty queries', async () => {
      const { result } = renderHook(() => useStreamingQuery());

      await act(async () => {
        await result.current.askOrbit('');
      });

      expect(invoke).not.toHaveBeenCalled();
      expect(result.current.history).toEqual([]);
    });

    it('should not process whitespace-only queries', async () => {
      const { result } = renderHook(() => useStreamingQuery());

      await act(async () => {
        await result.current.askOrbit('   ');
      });

      expect(invoke).not.toHaveBeenCalled();
      expect(result.current.history).toEqual([]);
    });

    it('should clear previous response when starting new query', async () => {
      const mockResponse1 = 'First response';
      const mockResponse2 = 'Second response';

      (invoke as any)
        .mockResolvedValueOnce(mockResponse1)
        .mockResolvedValueOnce(mockResponse2);

      const { result } = renderHook(() => useStreamingQuery());

      // First query
      await act(async () => {
        await result.current.askOrbit('First query');
      });

      expect(result.current.history).toHaveLength(2);

      // Second query
      await act(async () => {
        await result.current.askOrbit('Second query');
      });

      expect(result.current.history).toHaveLength(4);
      expect(result.current.history[2]).toEqual({ speaker: 'user', text: 'Second query' });
      expect(result.current.history[3]).toEqual({ speaker: 'ai', text: mockResponse2 });
    });

    it('should set loading state correctly', async () => {
      let resolvePromise: (value: any) => void;
      const promise = new Promise((resolve) => {
        resolvePromise = resolve;
      });

      (invoke as any).mockReturnValueOnce(promise);

      const { result } = renderHook(() => useStreamingQuery());

      expect(result.current.isLoading).toBe(false);

      act(() => {
        result.current.askOrbit('Test query');
      });

      expect(result.current.isLoading).toBe(true);

      await act(async () => {
        resolvePromise!('Response');
        await promise;
      });

      expect(result.current.isLoading).toBe(false);
    });
  });

  describe('setHistory', () => {
    it('should allow manual history updates', () => {
      const { result } = renderHook(() => useStreamingQuery());

      act(() => {
        result.current.setHistory([
          { speaker: 'user', text: 'Manual message' },
          { speaker: 'ai', text: 'Manual response' }
        ]);
      });

      expect(result.current.history).toEqual([
        { speaker: 'user', text: 'Manual message' },
        { speaker: 'ai', text: 'Manual response' }
      ]);
    });
  });

  describe('setIsLoading', () => {
    it('should allow manual loading state updates', () => {
      const { result } = renderHook(() => useStreamingQuery());

      expect(result.current.isLoading).toBe(false);

      act(() => {
        result.current.setIsLoading(true);
      });

      expect(result.current.isLoading).toBe(true);

      act(() => {
        result.current.setIsLoading(false);
      });

      expect(result.current.isLoading).toBe(false);
    });
  });
});