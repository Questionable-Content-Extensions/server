import { useEffect, useMemo, useState } from 'react';

import type { SocialHubEntry } from '../../../bindings/SocialHubEntry';
import ItemDetailsModal from './ItemDetailsModal';

type SortKey = 'partners' | 'appearances' | 'name';
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

export default function SocialHub() {
    const [data, setData] = useState<SocialHubEntry[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [sort, setSort] = useState<SortState>({
        key: 'partners',
        dir: 'desc',
    });
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/social-hub')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<SocialHubEntry[]>;
            })
            .then(setData)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    function handleSort(key: SortKey) {
        setSort((prev) => ({
            key,
            dir: prev.key === key && prev.dir === 'desc' ? 'asc' : 'desc',
        }));
    }

    const sorted = useMemo(() => {
        if (!data) return null;
        const copy = [...data];
        copy.sort((a, b) => {
            const diff =
                sort.key === 'partners'
                    ? a.distinctPartners - b.distinctPartners
                    : sort.key === 'appearances'
                      ? a.appearances - b.appearances
                      : a.name.localeCompare(b.name);
            return sort.dir === 'asc' ? diff : -diff;
        });
        return copy;
    }, [data, sort]);

    const maxPartners = useMemo(
        () => (data ? Math.max(...data.map((d) => d.distinctPartners)) : 1),
        [data],
    );

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
                    Social Hub
                </h2>
                <p className="text-sm text-gray-500 mb-4">
                    Characters ranked by the number of distinct other cast
                    members they have appeared with. A character with a high hub
                    score connects the most unique people in the comic&apos;s
                    social graph. Requires at least one shared comic with
                    another cast member.
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
                                    label="Distinct partners"
                                    sortKey="partners"
                                    current={sort}
                                    onSort={handleSort}
                                />
                                <SortHeader
                                    label="Appearances"
                                    sortKey="appearances"
                                    current={sort}
                                    onSort={handleSort}
                                />
                                <th className="py-2 font-medium text-right text-gray-600">
                                    Reach
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                            {sorted.map((row, i) => {
                                const reach =
                                    maxPartners > 0
                                        ? (row.distinctPartners / maxPartners) *
                                          100
                                        : 0;
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
                                        <td className="py-2 pr-4 text-right font-medium text-indigo-700">
                                            {row.distinctPartners.toLocaleString()}
                                        </td>
                                        <td className="py-2 pr-4 text-right text-gray-500">
                                            {row.appearances.toLocaleString()}
                                        </td>
                                        <td className="py-2 text-right text-gray-500">
                                            {reach.toFixed(0)}%
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
