import { useEffect, useState } from 'react';

import type { Item } from 'bindings/Item';
import type { ItemList } from 'bindings/ItemList';
import type { RelatedItem } from 'bindings/RelatedItem';
import { getItemdata } from 'bindings/api/GetItemdata';
import { getItemdataItemId } from 'bindings/api/GetItemdataItemId';
import { getItemdataItemIdFriends } from 'bindings/api/GetItemdataItemIdFriends';
import { getItemdataItemIdLocations } from 'bindings/api/GetItemdataItemIdLocations';

let _allItemsCache: Map<number, ItemList> | null = null;
let _allItemsFetch: Promise<Map<number, ItemList>> | null = null;

export function _resetItemsCache(): void {
    _allItemsCache = null;
    _allItemsFetch = null;
}

function getOrFetchAllItems(): Promise<Map<number, ItemList>> {
    if (_allItemsCache) return Promise.resolve(_allItemsCache);
    if (_allItemsFetch) return _allItemsFetch;
    _allItemsFetch = getItemdata()
        .then((items) => {
            _allItemsCache = new Map(items.map((i) => [i.id, i]));
            return _allItemsCache;
        })
        .finally(() => {
            _allItemsFetch = null;
        });
    return _allItemsFetch;
}

function comicLink(comicId: number): string {
    return `https://questionablecontent.net/view.php?comic=${comicId}`;
}

interface ItemDetailsModalProps {
    initialItemId: number;
    onClose: () => void;
}

export default function ItemDetailsModal({
    initialItemId,
    onClose,
}: ItemDetailsModalProps) {
    const [prevInitialItemId, setPrevInitialItemId] =
        useState<number>(initialItemId);
    const [currentItemId, setCurrentItemId] = useState<number>(initialItemId);

    if (prevInitialItemId !== initialItemId) {
        setPrevInitialItemId(initialItemId);
        setCurrentItemId(initialItemId);
    }

    const [loadedForItemId, setLoadedForItemId] = useState<number | null>(null);
    const [itemData, setItemData] = useState<Item | null>(null);
    const [friends, setFriends] = useState<RelatedItem[]>([]);
    const [locations, setLocations] = useState<RelatedItem[]>([]);
    const [allItems, setAllItems] = useState<Map<number, ItemList> | null>(
        null,
    );
    const [fetchError, setFetchError] = useState<string | null>(null);

    // Derived: still waiting for the current item to finish loading
    const loading = loadedForItemId !== currentItemId;
    // Only surface an error that belongs to the current item
    const error = loadedForItemId === currentItemId ? fetchError : null;

    useEffect(() => {
        getOrFetchAllItems()
            .then((m) => setAllItems(m))
            .catch(() => {});
    }, []);

    useEffect(() => {
        const controller = new AbortController();
        const { signal } = controller;

        getItemdataItemId(currentItemId, { signal })
            .then((data) => {
                setItemData(data);
                setFriends([]);
                setLocations([]);
                setFetchError(null);
                setLoadedForItemId(currentItemId);
                return Promise.all([
                    getItemdataItemIdFriends(currentItemId, { signal })
                        .then(setFriends)
                        .catch(() => {}),
                    getItemdataItemIdLocations(currentItemId, { signal })
                        .then(setLocations)
                        .catch(() => {}),
                ]);
            })
            .catch((e: unknown) => {
                if (e instanceof DOMException && e.name === 'AbortError')
                    return;
                setFetchError(e instanceof Error ? e.message : String(e));
                setLoadedForItemId(currentItemId);
            });

        return () => {
            controller.abort();
        };
    }, [currentItemId]);

    useEffect(() => {
        const handler = (e: KeyboardEvent) => {
            if (e.key === 'Escape') onClose();
        };
        document.addEventListener('keydown', handler);
        return () => document.removeEventListener('keydown', handler);
    }, [onClose]);

    const typeLabel =
        itemData?.type === 'cast'
            ? 'Character'
            : itemData?.type === 'location'
              ? 'Location'
              : itemData?.type === 'storyline'
                ? 'Storyline'
                : null;

    const imageUrl =
        itemData?.primaryImage != null
            ? `/api/v3/itemdata/image/${itemData.primaryImage}`
            : null;

    function renderRelatedList(items: RelatedItem[], heading: string) {
        if (items.length === 0) return null;
        return (
            <div className="mt-4">
                <h4 className="text-sm font-semibold text-gray-700 mb-1">
                    {heading}
                </h4>
                <ul className="space-y-1">
                    {items.map((rel) => {
                        const name =
                            allItems?.get(rel.id)?.name ?? `#${rel.id}`;
                        return (
                            <li
                                key={rel.id}
                                className="flex items-center justify-between text-sm"
                            >
                                <button
                                    type="button"
                                    onClick={() => {
                                        setCurrentItemId(rel.id);
                                    }}
                                    className="text-blue-600 hover:underline text-left"
                                >
                                    {name}
                                </button>
                                <span className="text-gray-400 ml-3 text-xs tabular-nums">
                                    {rel.count.toLocaleString()} shared
                                </span>
                            </li>
                        );
                    })}
                </ul>
            </div>
        );
    }

    return (
        <div
            className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
            onClick={onClose}
        >
            <div
                className="bg-white rounded-lg shadow-xl w-full max-w-md mx-4 max-h-[90vh] flex flex-col"
                onClick={(e) => {
                    e.stopPropagation();
                }}
            >
                <div className="flex items-start justify-between p-4 border-b border-gray-200">
                    <div>
                        <h3 className="text-lg font-semibold text-gray-900 leading-tight">
                            {loading
                                ? 'Loading…'
                                : error
                                  ? 'Error'
                                  : (itemData?.name ?? '…')}
                        </h3>
                        {typeLabel && (
                            <span className="text-xs font-medium text-gray-500">
                                {typeLabel}
                            </span>
                        )}
                    </div>
                    <button
                        type="button"
                        onClick={onClose}
                        className="ml-4 shrink-0 text-gray-400 hover:text-gray-600 text-xl leading-none"
                        aria-label="Close dialog"
                    >
                        ×
                    </button>
                </div>

                <div className="p-4 overflow-y-auto flex-1">
                    {error && (
                        <p className="text-red-600 text-sm">
                            Failed to load: {error}
                        </p>
                    )}
                    {loading && !error && (
                        <p className="text-gray-500 text-sm">Loading…</p>
                    )}
                    {!loading && !error && itemData && (
                        <>
                            <div className="flex gap-4">
                                {imageUrl && (
                                    <img
                                        src={imageUrl}
                                        alt={itemData.name}
                                        className="max-h-32 w-auto rounded shrink-0"
                                    />
                                )}
                                <dl className="text-sm grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 items-baseline">
                                    <dt className="text-gray-500">
                                        Appearances
                                    </dt>
                                    <dd className="font-medium text-gray-900">
                                        {itemData.appearances.toLocaleString()}
                                    </dd>
                                    <dt className="text-gray-500">Presence</dt>
                                    <dd className="font-medium text-gray-900">
                                        {itemData.presence.toFixed(1)}%
                                    </dd>
                                    <dt className="text-gray-500">
                                        First comic
                                    </dt>
                                    <dd>
                                        <a
                                            href={comicLink(itemData.first)}
                                            target="_blank"
                                            rel="noreferrer"
                                            className="text-blue-600 hover:underline"
                                        >
                                            #{itemData.first}
                                        </a>
                                    </dd>
                                    <dt className="text-gray-500">
                                        Last comic
                                    </dt>
                                    <dd>
                                        <a
                                            href={comicLink(itemData.last)}
                                            target="_blank"
                                            rel="noreferrer"
                                            className="text-blue-600 hover:underline"
                                        >
                                            #{itemData.last}
                                        </a>
                                    </dd>
                                </dl>
                            </div>

                            {renderRelatedList(
                                friends,
                                itemData.type === 'location'
                                    ? 'Top characters'
                                    : 'Top co-stars',
                            )}
                            {itemData.type === 'cast' &&
                                renderRelatedList(locations, 'Top locations')}
                        </>
                    )}
                </div>

                <div className="p-4 border-t border-gray-200 flex justify-end">
                    <button
                        type="button"
                        onClick={onClose}
                        className="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 rounded"
                    >
                        Close
                    </button>
                </div>
            </div>
        </div>
    );
}
