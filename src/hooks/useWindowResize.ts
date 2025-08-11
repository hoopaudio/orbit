import { useEffect, RefObject } from 'react';
import { invoke } from "@tauri-apps/api/core";

export const useWindowResize = (
    containerRef: RefObject<HTMLDivElement>,
    response: string,
    isLoading: boolean
) => {
    useEffect(() => {
        if (!containerRef.current) return;
        
        // Use a small delay to ensure DOM has updated after content change
        setTimeout(() => {
            if (!containerRef.current) return;
            
            const containerHeight = containerRef.current.scrollHeight;
            // Add more padding: 25px top padding + 40px bottom padding + extra buffer
            const windowHeight = Math.min(Math.max(containerHeight + 80, 120), 800);
            
            invoke("resize_window", {
                width: 550,
                height: windowHeight
            }).catch(console.error);
        }, 100);
    }, [response, isLoading]);
};