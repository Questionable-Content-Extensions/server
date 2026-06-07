import { useEffect, useMemo, useState } from 'react';

import type { NeverMetPair } from '../../../bindings/NeverMetPair';
import ItemDetailsModal from './ItemDetailsModal';

type SortKey = 'together' | 'char1' | 'char2';
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

export default function NeverMet() {
    const [data, setData] = useState<NeverMetPair[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [sort, setSort] = useState<SortState>({
        key: 'together',
        dir: 'asc',
    });
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/never-met')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<NeverMetPair[]>;
            })
            .then(setData)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    function handleSort(key: SortKey) {
        setSort((prev) => ({
            key,
            dir: prev.key === key && prev.dir === 'asc' ? 'desc' : 'asc',
        }));
    }

    const sorted = useMemo(() => {
        if (!data) return null;
        const copy = [...data];
        copy.sort((a, b) => {
            const diff =
                sort.key === 'together'
                    ? a.comicsTogether - b.comicsTogether
                    : sort.key === 'char2'
                      ? a.character2Name.localeCompare(b.character2Name)
                      : a.character1Name.localeCompare(b.character1Name);
            return sort.dir === 'asc' ? diff : -diff;
        });
        return copy;
    }, [data, sort]);

    if (error)
        return <p className="text-red-600">Failed to load data: {error}</p>;
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
                    Never Met
                </h2>
                <p className="text-sm text-gray-500 mb-4">
                    Prominent character pairs who have rarely or never shared a
                    comic together, despite both being frequent cast members.
                    Drawn from the top 100 characters by appearances. A
                    &quot;comics together&quot; count of 0 means they have
                    literally never appeared in the same comic. Only pairs with
                    2 or fewer shared comics are shown.
                </p>
                <div className="overflow-x-auto">
                    <table className="min-w-full text-sm">
                        <thead>
                            <tr className="border-b border-gray-200 text-left text-gray-600">
                                <th className="py-2 pr-4 font-medium w-12">
                                    #
                                </th>
                                <SortHeader
                                    label="Character 1"
                                    sortKey="char1"
                                    current={sort}
                                    onSort={handleSort}
                                    align="left"
                                />
                                <th className="py-2 pr-4 font-medium text-right text-gray-600">
                                    Appearances
                                </th>
                                <SortHeader
                                    label="Character 2"
                                    sortKey="char2"
                                    current={sort}
                                    onSort={handleSort}
                                    align="left"
                                />
                                <th className="py-2 pr-4 font-medium text-right text-gray-600">
                                    Appearances
                                </th>
                                <SortHeader
                                    label="Together"
                                    sortKey="together"
                                    current={sort}
                                    onSort={handleSort}
                                />
                            </tr>
                        </thead>
                        <tbody>
                            {sorted.map((row, i) => (
                                <tr
                                    key={`${row.character1Id}-${row.character2Id}`}
                                    className="border-b border-gray-100 hover:bg-gray-50"
                                >
                                    <td className="py-2 pr-4 text-gray-400">
                                        {i + 1}
                                    </td>
                                    <td className="py-2 pr-4">
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setSelectedItemId(
                                                    row.character1Id,
                                                );
                                            }}
                                            className="font-medium text-gray-900 hover:text-blue-600 hover:underline text-left"
                                        >
                                            {row.character1Name}
                                        </button>
                                    </td>
                                    <td className="py-2 pr-4 text-right text-gray-500">
                                        {row.character1Appearances.toLocaleString()}
                                    </td>
                                    <td className="py-2 pr-4">
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setSelectedItemId(
                                                    row.character2Id,
                                                );
                                            }}
                                            className="font-medium text-gray-900 hover:text-blue-600 hover:underline text-left"
                                        >
                                            {row.character2Name}
                                        </button>
                                    </td>
                                    <td className="py-2 pr-4 text-right text-gray-500">
                                        {row.character2Appearances.toLocaleString()}
                                    </td>
                                    <td
                                        className={`py-2 text-right font-medium ${row.comicsTogether === 0 ? 'text-red-600' : 'text-orange-500'}`}
                                    >
                                        {row.comicsTogether === 0
                                            ? 'Never'
                                            : row.comicsTogether.toLocaleString()}
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            </div>
        </>
    );
}
