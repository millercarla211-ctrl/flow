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

  const persist = useCallback(
    (nextItems: T[]) => {
      setItems(nextItems);
      window.localStorage.setItem(key, JSON.stringify(nextItems));
      emitStorageChange(key);
    },
    [key],
  );

  const addItem = useCallback(
    (item: T) => {
      persist([item, ...items]);
    },
    [items, persist],
  );

  const updateItem = useCallback(
    (id: string, update: Partial<T>) => {
      const now = new Date().toISOString();
      persist(
        items.map((item) =>
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
    [items, persist],
  );

  const updateWhere = useCallback(
    (predicate: (item: T) => boolean, update: Partial<T>) => {
      const now = new Date().toISOString();
      persist(
        items.map((item) =>
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
    [items, persist],
  );

  const removeItem = useCallback(
    (id: string) => {
      persist(items.filter((item) => item.id !== id));
    },
    [items, persist],
  );

  const removeWhere = useCallback(
    (predicate: (item: T) => boolean) => {
      persist(items.filter((item) => !predicate(item)));
    },
    [items, persist],
  );

  return { items, isLoaded, addItem, updateItem, updateWhere, removeItem, removeWhere };
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
