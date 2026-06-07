import { useEffect, useState } from 'react';

import type { LocationCoOccurrenceResponse } from '../../../bindings/LocationCoOccurrenceResponse';
import ItemDetailsModal from './ItemDetailsModal';

function pct(together: number, appearances: number) {
    if (appearances === 0) return '—';
    return `${((together / appearances) * 100).toFixed(1)}%`;
}

export default function LocationCoOccurrences() {
    const [data, setData] = useState<LocationCoOccurrenceResponse | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [selectedItemId, setSelectedItemId] = useState<number | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/location-co-occurrences')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<LocationCoOccurrenceResponse>;
            })
            .then(setData)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    if (error) {
        return <p className="text-red-600">Failed to load data: {error}</p>;
    }

    if (!data) {
        return <p className="text-gray-500">Loading…</p>;
    }

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
                    Location Co-Occurrences
                </h2>
                <p className="text-sm text-gray-500 mb-4">
                    The top 50 location pairs by how often they appear in the
                    same comic. The % columns show what share of each location's
                    total appearances include the other.
                </p>
                <div className="overflow-x-auto">
                    <table className="min-w-full text-sm">
                        <thead>
                            <tr className="border-b border-gray-200 text-left text-gray-600">
                                <th className="py-2 pr-4 font-medium w-12">
                                    #
                                </th>
                                <th className="py-2 pr-4 font-medium">
                                    Location 1
                                </th>
                                <th className="py-2 pr-4 font-medium">
                                    Location 2
                                </th>
                                <th className="py-2 pr-4 font-medium text-right">
                                    Comics together
                                </th>
                                <th className="py-2 pr-4 font-medium text-right">
                                    % of L1
                                </th>
                                <th className="py-2 font-medium text-right">
                                    % of L2
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                            {data.pairs.map((pair, i) => {
                                const l1 = data.locations[pair.location1Id];
                                const l2 = data.locations[pair.location2Id];
                                return (
                                    <tr
                                        key={`${pair.location1Id}-${pair.location2Id}`}
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
                                                        pair.location1Id,
                                                    );
                                                }}
                                                className="font-medium text-gray-900 hover:text-blue-600 hover:underline text-left"
                                            >
                                                {l1?.name}
                                            </button>
                                        </td>
                                        <td className="py-2 pr-4">
                                            <button
                                                type="button"
                                                onClick={() => {
                                                    setSelectedItemId(
                                                        pair.location2Id,
                                                    );
                                                }}
                                                className="font-medium text-gray-900 hover:text-blue-600 hover:underline text-left"
                                            >
                                                {l2?.name}
                                            </button>
                                        </td>
                                        <td className="py-2 pr-4 text-right text-gray-700">
                                            {pair.comicsTogether.toLocaleString()}
                                        </td>
                                        <td className="py-2 pr-4 text-right text-gray-500">
                                            {l1
                                                ? pct(
                                                      pair.comicsTogether,
                                                      l1.appearances,
                                                  )
                                                : '—'}
                                        </td>
                                        <td className="py-2 text-right text-gray-500">
                                            {l2
                                                ? pct(
                                                      pair.comicsTogether,
                                                      l2.appearances,
                                                  )
                                                : '—'}
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
