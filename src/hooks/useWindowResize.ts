import {useEffect, RefObject} from 'react';
import {invoke} from "@tauri-apps/api/core";

export const useWindowResize = (
    containerRef: RefObject<HTMLDivElement>,
    response: string,
    isLoading: boolean,
    isProducerMode: boolean = false
) => {
    useEffect(() => {
        // Skip resize in producer mode as it's handled separately
        if (!containerRef.current || isProducerMode) return;

        // Use a small delay to ensure DOM has updated after content change
        setTimeout(() => {
            if (!containerRef.current) return;

            const containerHeight = containerRef.current.scrollHeight;
            // Add more padding: 25px top padding + 40px bottom padding + extra buffer
            const windowHeight = Math.min(Math.max(containerHeight + 60, 185), 1050);

            invoke("resize_window", {
                width: 550,
                height: windowHeight
            }).catch(console.error);
        }, 100);
    }, [response, isLoading]);
};