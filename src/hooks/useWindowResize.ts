import { useEffect, RefObject } from 'react';
import { invoke } from "@tauri-apps/api/core";

export const useWindowResize = (
    containerRef: RefObject<HTMLDivElement>,
    response: string,
    isLoading: boolean
) => {
    useEffect(() => {
        if (!containerRef.current) return;
        
        const containerHeight = containerRef.current.scrollHeight;
        const windowHeight = Math.min(Math.max(containerHeight + 40, 120), 600);
        
        invoke("resize_window", {
            width: 550,
            height: windowHeight
        }).catch(console.error);
    }, [response, isLoading]);
};