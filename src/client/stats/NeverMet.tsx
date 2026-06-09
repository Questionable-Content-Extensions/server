import { useEffect, useMemo, useState } from 'react';

import type { NeverMetPair } from 'bindings/NeverMetPair';
import { getStatsNeverMet } from 'bindings/api/GetStatsNeverMet';

import ItemDetailsModal from './ItemDetailsModal';
import {
    SortableHeader,
    StaticHeader,
    StatsTable,
    StatsTbodyRow,
    StatsTheadRow,
    useSortState,
} from './StatsTable';

type SortKey = 'together' | 'char1' | 'char2';

export default function NeverMet() {
    const [data, setData] = useState<NeverMetPair[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [sort, handleSort] = useSortState<SortKey>('together', 'asc');
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);

    useEffect(() => {
        getStatsNeverMet()
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
                <StatsTable>
                    <thead>
                        <StatsTheadRow>
                            <StaticHeader className="w-12">#</StaticHeader>
                            <SortableHeader
                                sortKey="char1"
                                sort={sort}
                                onSort={handleSort}
                                align="left"
                            >
                                Character 1
                            </SortableHeader>
                            <StaticHeader align="right">
                                Appearances
                            </StaticHeader>
                            <SortableHeader
                                sortKey="char2"
                                sort={sort}
                                onSort={handleSort}
                                align="left"
                            >
                                Character 2
                            </SortableHeader>
                            <StaticHeader align="right">
                                Appearances
                            </StaticHeader>
                            <SortableHeader
                                sortKey="together"
                                sort={sort}
                                onSort={handleSort}
                            >
                                Together
                            </SortableHeader>
                        </StatsTheadRow>
                    </thead>
                    <tbody>
                        {sorted.map((row, i) => (
                            <StatsTbodyRow
                                key={`${row.character1Id}-${row.character2Id}`}
                            >
                                <td className="py-2 pr-4 text-gray-400">
                                    {i + 1}
                                </td>
                                <td className="py-2 pr-4">
                                    <button
                                        type="button"
                                        onClick={() => {
                                            setSelectedItemId(row.character1Id);
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
                                            setSelectedItemId(row.character2Id);
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
                            </StatsTbodyRow>
                        ))}
                    </tbody>
                </StatsTable>
            </div>
        </>
    );
}
