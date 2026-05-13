"use client";

import { useCallback, useEffect, useState } from "react";

const FRIDAY_STORAGE_EVENT = "friday-local-storage-changed";

function emitStorageChange(key: string) {
  window.dispatchEvent(new CustomEvent(FRIDAY_STORAGE_EVENT, { detail: { key } }));
}

function safeJsonParse<T>(raw: string | null, fallback: T): T {
  if (!raw) return fallback;
  try {
    return JSON.parse(raw) as T;
  } catch {
    return fallback;
  }
}

function createId(prefix: string) {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return `${prefix}_${crypto.randomUUID()}`;
  }
  return `${prefix}_${Date.now()}_${Math.random().toString(16).slice(2)}`;
}

export type LocalRecord = {
  id: string;
  createdAt: string;
  updatedAt: string;
};

export function makeLocalRecord<T extends object>(prefix: string, value: T): T & LocalRecord {
  const now = new Date().toISOString();
  return {
    ...value,
    id: createId(prefix),
    createdAt: now,
    updatedAt: now,
  };
}

export function useLocalList<T extends LocalRecord>(key: string) {
  const [items, setItems] = useState<T[]>([]);
  const [isLoaded, setIsLoaded] = useState(false);

  const load = useCallback(() => {
    setItems(safeJsonParse<T[]>(window.localStorage.getItem(key), []));
    setIsLoaded(true);
  }, [key]);

  useEffect(() => {
    load();

    const onChange = (event: Event) => {
      const detail = (event as CustomEvent<{ key?: string }>).detail;
      if (!detail?.key || detail.key === key) {
        load();
      }
    };

    window.addEventListener(FRIDAY_STORAGE_EVENT, onChange);
    return () => window.removeEventListener(FRIDAY_STORAGE_EVENT, onChange);
  }, [key, load]);

  const readCurrentItems = useCallback(
    () => safeJsonParse<T[]>(window.localStorage.getItem(key), []),
    [key],
  );

  const persist = useCallback(
    (nextItems: T[]) => {
      setItems(nextItems);
      window.localStorage.setItem(key, JSON.stringify(nextItems));
      emitStorageChange(key);
    },
    [key],
  );

  const updateItems = useCallback(
    (updater: (currentItems: T[]) => T[]) => {
      persist(updater(readCurrentItems()));
    },
    [persist, readCurrentItems],
  );

  const addItem = useCallback(
    (item: T) => {
      updateItems((currentItems) => [item, ...currentItems]);
    },
    [updateItems],
  );

  const addItems = useCallback(
    (nextItems: T[]) => {
      if (nextItems.length === 0) return;
      updateItems((currentItems) => [...nextItems, ...currentItems]);
    },
    [updateItems],
  );

  const updateItem = useCallback(
    (id: string, update: Partial<T>) => {
      const now = new Date().toISOString();
      updateItems((currentItems) =>
        currentItems.map((item) =>
          item.id === id
            ? {
                ...item,
                ...update,
                updatedAt: now,
              }
            : item,
        ),
      );
    },
    [updateItems],
  );

  const updateWhere = useCallback(
    (predicate: (item: T) => boolean, update: Partial<T>) => {
      const now = new Date().toISOString();
      updateItems((currentItems) =>
        currentItems.map((item) =>
          predicate(item)
            ? {
                ...item,
                ...update,
                updatedAt: now,
              }
            : item,
        ),
      );
    },
    [updateItems],
  );

  const removeItem = useCallback(
    (id: string) => {
      updateItems((currentItems) => currentItems.filter((item) => item.id !== id));
    },
    [updateItems],
  );

  const removeWhere = useCallback(
    (predicate: (item: T) => boolean) => {
      updateItems((currentItems) => currentItems.filter((item) => !predicate(item)));
    },
    [updateItems],
  );

  return { items, isLoaded, addItem, addItems, updateItem, updateWhere, removeItem, removeWhere };
}

export function useLocalSettings<T extends object>(key: string, defaults: T) {
  const [settings, setSettings] = useState<T>(defaults);
  const [isLoaded, setIsLoaded] = useState(false);

  const load = useCallback(() => {
    setSettings(safeJsonParse<T>(window.localStorage.getItem(key), defaults));
    setIsLoaded(true);
  }, [defaults, key]);

  useEffect(() => {
    load();

    const onChange = (event: Event) => {
      const detail = (event as CustomEvent<{ key?: string }>).detail;
      if (!detail?.key || detail.key === key) {
        load();
      }
    };

    window.addEventListener(FRIDAY_STORAGE_EVENT, onChange);
    return () => window.removeEventListener(FRIDAY_STORAGE_EVENT, onChange);
  }, [key, load]);

  const updateSettings = useCallback(
    (update: Partial<T>) => {
      const nextSettings = { ...settings, ...update };
      setSettings(nextSettings);
      window.localStorage.setItem(key, JSON.stringify(nextSettings));
      emitStorageChange(key);
    },
    [key, settings],
  );

  return { settings, isLoaded, updateSettings };
}
