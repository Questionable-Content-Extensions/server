import { useMemo, useState } from 'react';

import type { ItemStats } from '../../../bindings/ItemStats';
import ItemDetailsModal from './ItemDetailsModal';

type SortKey = 'span' | 'appearances' | 'first' | 'last' | 'name';
type SortDir = 'asc' | 'desc';

interface SortState {
    key: SortKey;
    dir: SortDir;
}

function SortHeader({
    label,
    sortKey,
    current,
    onSort,
    align = 'right',
}: {
    label: string;
    sortKey: SortKey;
    current: SortState;
    onSort: (key: SortKey) => void;
    align?: 'left' | 'right';
}) {
    const isActive = current.key === sortKey;
    const arrow = isActive ? (current.dir === 'asc' ? ' ↑' : ' ↓') : '';
    return (
        <th
            className={`py-2 pr-4 font-medium cursor-pointer select-none hover:text-gray-900 ${align === 'right' ? 'text-right' : 'text-left'} ${isActive ? 'text-gray-900' : ''}`}
            onClick={() => {
                onSort(sortKey);
            }}
        >
            {label}
            {arrow}
        </th>
    );
}

interface Props {
    locationsData: ItemStats[] | null;
    locationsError: string | null;
}

export default function LocationLifespan({
    locationsData,
    locationsError,
}: Props) {
    const [sort, setSort] = useState<SortState>({ key: 'span', dir: 'desc' });
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);

    function handleSort(key: SortKey) {
        setSort((prev) => ({
            key,
            dir: prev.key === key && prev.dir === 'desc' ? 'asc' : 'desc',
        }));
    }

    const sorted = useMemo(() => {
        if (!locationsData) return null;
        const copy = [...locationsData];
        copy.sort((a, b) => {
            const spanA = a.lastComic - a.firstComic;
            const spanB = b.lastComic - b.firstComic;
            const diff =
                sort.key === 'span'
                    ? spanA - spanB
                    : sort.key === 'appearances'
                      ? a.appearances - b.appearances
                      : sort.key === 'first'
                        ? a.firstComic - b.firstComic
                        : sort.key === 'last'
                          ? a.lastComic - b.lastComic
                          : a.name.localeCompare(b.name);
            return sort.dir === 'asc' ? diff : -diff;
        });
        return copy;
    }, [locationsData, sort]);

    if (locationsError)
        return (
            <p className="text-red-600">
                Failed to load data: {locationsError}
            </p>
        );
    if (!sorted) return <p className="text-gray-500">Loading…</p>;

    return (
        <>
            {selectedItemId !== null && (
                <ItemDetailsModal
                    initialItemId={selectedItemId}
                    onClose={() => {
                        setSelectedItemId(null);
                    }}
                />
            )}
            <div>
                <h2 className="text-xl font-semibold text-gray-800 mb-1">
                    Location Lifespan
                </h2>
                <p className="text-sm text-gray-500 mb-4">
                    Every location ranked by how long it has been in use,
                    measured as the comic-number span between its first and last
                    appearance. A wide span with few appearances indicates a
                    location used only occasionally over a long period.
                </p>
                <div className="overflow-x-auto">
                    <table className="min-w-full text-sm">
                        <thead>
                            <tr className="border-b border-gray-200 text-left text-gray-600">
                                <th className="py-2 pr-4 font-medium w-12">
                                    #
                                </th>
                                <SortHeader
                                    label="Name"
                                    sortKey="name"
                                    current={sort}
                                    onSort={handleSort}
                                    align="left"
                                />
                                <SortHeader
                                    label="First comic"
                                    sortKey="first"
                                    current={sort}
                                    onSort={handleSort}
                                />
                                <SortHeader
                                    label="Last comic"
                                    sortKey="last"
                                    current={sort}
                                    onSort={handleSort}
                                />
                                <SortHeader
                                    label="Comic span"
                                    sortKey="span"
                                    current={sort}
                                    onSort={handleSort}
                                />
                                <SortHeader
                                    label="Appearances"
                                    sortKey="appearances"
                                    current={sort}
                                    onSort={handleSort}
                                />
                            </tr>
                        </thead>
                        <tbody>
                            {sorted.map((row, i) => {
                                const span = row.lastComic - row.firstComic;
                                return (
                                    <tr
                                        key={row.id}
                                        className="border-b border-gray-100 hover:bg-gray-50"
                                    >
                                        <td className="py-2 pr-4 text-gray-400">
                                            {i + 1}
                                        </td>
                                        <td className="py-2 pr-4">
                                            <button
                                                type="button"
                                                onClick={() => {
                                                    setSelectedItemId(row.id);
                                                }}
                                                className="font-medium text-gray-900 hover:text-blue-600 hover:underline text-left"
                                            >
                                                {row.name}
                                            </button>
                                        </td>
                                        <td className="py-2 pr-4 text-right text-gray-500">
                                            #{row.firstComic}
                                        </td>
                                        <td className="py-2 pr-4 text-right text-gray-500">
                                            #{row.lastComic}
                                        </td>
                                        <td className="py-2 pr-4 text-right font-medium text-indigo-700">
                                            {span.toLocaleString()}
                                        </td>
                                        <td className="py-2 text-right text-gray-500">
                                            {row.appearances.toLocaleString()}
                                        </td>
                                    </tr>
                                );
                            })}
                        </tbody>
                    </table>
                </div>
            </div>
        </>
    );
}
