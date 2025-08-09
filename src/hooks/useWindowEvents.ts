import { useEffect, RefObject } from 'react';
import { getCurrentWindow } from "@tauri-apps/api/window";

export const useWindowEvents = (
    inputRef: RefObject<HTMLTextAreaElement>,
    setQuery: (query: string) => void,
    setResponse: (response: string) => void,
    setIsLoading: (loading: boolean) => void
) => {
    useEffect(() => {
        const setupListeners = async () => {
            if (inputRef.current) {
                inputRef.current.focus();
            }

            const unlisten = await getCurrentWindow().onFocusChanged(({payload: focused}) => {
                if (focused) {
                    setQuery("");
                    setResponse("");
                    setIsLoading(false);
                }
            });

            return unlisten;
        };

        setupListeners();
    }, [setQuery, setResponse, setIsLoading]);
};