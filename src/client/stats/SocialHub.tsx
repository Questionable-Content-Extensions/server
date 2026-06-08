import { useEffect, useMemo, useState } from 'react';

import type { SocialHubEntry } from '../../../bindings/SocialHubEntry';
import ItemDetailsModal from './ItemDetailsModal';
import {
    SortableHeader,
    StaticHeader,
    StatsTable,
    StatsTbodyRow,
    StatsTheadRow,
    useSortState,
} from './StatsTable';

type SortKey = 'partners' | 'appearances' | 'name';

export default function SocialHub() {
    const [data, setData] = useState<SocialHubEntry[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [sort, handleSort] = useSortState<SortKey>('partners', 'desc');
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
                <StatsTable>
                    <thead>
                        <StatsTheadRow>
                            <StaticHeader className="w-12">#</StaticHeader>
                            <SortableHeader
                                sortKey="name"
                                sort={sort}
                                onSort={handleSort}
                                align="left"
                            >
                                Name
                            </SortableHeader>
                            <SortableHeader
                                sortKey="partners"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Distinct partners
                            </SortableHeader>
                            <SortableHeader
                                sortKey="appearances"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Appearances
                            </SortableHeader>
                            <StaticHeader align="right">Reach</StaticHeader>
                        </StatsTheadRow>
                    </thead>
                    <tbody>
                        {sorted.map((row, i) => {
                            const reach =
                                maxPartners > 0
                                    ? (row.distinctPartners / maxPartners) * 100
                                    : 0;
                            return (
                                <StatsTbodyRow key={row.id}>
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
                                </StatsTbodyRow>
                            );
                        })}
                    </tbody>
                </StatsTable>
            </div>
        </>
    );
}
