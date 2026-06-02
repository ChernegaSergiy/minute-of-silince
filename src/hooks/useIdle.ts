import { useState, useEffect } from "react";

/**
 * Hook to detect user inactivity (idle state) based on mouse, keyboard, and touch events.
 * Extracted and optimized from react-use's useIdle with an inline throttle.
 *
 * @param timeoutMs The duration of inactivity in milliseconds before entering idle state.
 * @returns A boolean indicating if the user is currently idle.
 */
export function useIdle(timeoutMs = 15000): boolean {
  const [isIdle, setIsIdle] = useState(false);

  useEffect(() => {
    let timerId: number;
    let mounted = true;
    let lastEventTime = 0;

    const setIdleState = (idle: boolean) => {
      if (mounted) {
        setIsIdle(idle);
      }
    };

    const handleActivity = () => {
      const now = Date.now();
      // Throttle: process events at most once every 100ms
      if (now - lastEventTime < 100) return;
      lastEventTime = now;

      setIdleState(false);
      window.clearTimeout(timerId);
      timerId = window.setTimeout(() => setIdleState(true), timeoutMs);
    };

    const handleVisibilityChange = () => {
      if (!document.hidden) {
        handleActivity();
      }
    };

    // Start initial timer
    timerId = window.setTimeout(() => setIdleState(true), timeoutMs);

    const events = [
      "mousemove",
      "mousedown",
      "resize",
      "keydown",
      "touchstart",
      "wheel",
    ];

    events.forEach((event) => window.addEventListener(event, handleActivity));
    document.addEventListener("visibilitychange", handleVisibilityChange);

    return () => {
      mounted = false;
      window.clearTimeout(timerId);
      events.forEach((event) => window.removeEventListener(event, handleActivity));
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  }, [timeoutMs]);

  return isIdle;
}
