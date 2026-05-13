"use client";

import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef } from "react";

import { useFridayAutomationRunner } from "../hooks/useFridayAutomationRunner";
import { selectNextDueAutomation } from "../utils/localAutomation";

const AUTOMATION_SCAN_INTERVAL_MS = 30_000;
const AUTOMATION_FIRST_SCAN_DELAY_MS = 2_000;

export function FridayAutomationRuntime() {
  const windowLabel = getCurrentWindow().label;
  const { isLoaded, items, runAutomation, runningAutomationId } = useFridayAutomationRunner();
  const hasScheduledInitialScan = useRef(false);

  useEffect(() => {
    if (windowLabel !== "main" || !isLoaded) return;

    const runNextDueAutomation = () => {
      if (runningAutomationId) return;
      const nextDueAutomation = selectNextDueAutomation(items);
      if (nextDueAutomation) void runAutomation(nextDueAutomation, "Scheduled");
    };

    const firstScanDelay = hasScheduledInitialScan.current
      ? AUTOMATION_SCAN_INTERVAL_MS
      : AUTOMATION_FIRST_SCAN_DELAY_MS;
    hasScheduledInitialScan.current = true;

    const firstScan = window.setTimeout(runNextDueAutomation, firstScanDelay);
    const interval = window.setInterval(runNextDueAutomation, AUTOMATION_SCAN_INTERVAL_MS);

    return () => {
      window.clearTimeout(firstScan);
      window.clearInterval(interval);
    };
  }, [isLoaded, items, runAutomation, runningAutomationId, windowLabel]);

  return null;
}
