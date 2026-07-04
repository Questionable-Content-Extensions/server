import type { MonthlyHeatmapEntry } from 'models/MonthlyHeatmapEntry';
import { useEffect, useMemo, useState } from 'react';

import { getStatsMonthlyHeatmap } from 'bindings/api/GetStatsMonthlyHeatmap';

const MONTH_LABELS = [
    'Jan',
    'Feb',
    'Mar',
    'Apr',
    'May',
    'Jun',
    'Jul',
    'Aug',
    'Sep',
    'Oct',
    'Nov',
    'Dec',
];

function heatColor(value: number, max: number): string {
    if (max === 0) return 'bg-gray-100';
    const ratio = value / max;
    if (ratio === 0) return 'bg-gray-100';
    if (ratio < 0.2) return 'bg-indigo-100';
    if (ratio < 0.4) return 'bg-indigo-200';
    if (ratio < 0.6) return 'bg-indigo-300';
    if (ratio < 0.8) return 'bg-indigo-400';
    return 'bg-indigo-600';
}

function textColor(value: number, max: number): string {
    if (max === 0) return 'text-gray-400';
    const ratio = value / max;
    return ratio >= 0.6 ? 'text-white' : 'text-gray-700';
}

export default function MonthlyHeatmap() {
    const [data, setData] = useState<MonthlyHeatmapEntry[] | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        getStatsMonthlyHeatmap()
            .then(setData)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    const { years, grid, maxComics } = useMemo(() => {
        if (!data)
            return {
                years: [],
                grid: new Map<number, number[]>(),
                maxComics: 0,
            };

        const yearSet = new Set(data.map((d) => d.year));
        const sortedYears = [...yearSet].sort((a, b) => a - b);
        const g = new Map<number, number[]>();
        for (const y of sortedYears) {
            g.set(y, new Array(12).fill(0) as number[]);
        }
        let max = 0;
        for (const row of data) {
            const arr = g.get(row.year);
            if (arr) {
                arr[row.month - 1] = row.comics;
                if (row.comics > max) max = row.comics;
            }
        }
        return { years: sortedYears, grid: g, maxComics: max };
    }, [data]);

    if (error)
        return <p className="text-red-600">Failed to load data: {error}</p>;
    if (!data) return <p className="text-gray-500">Loading…</p>;

    return (
        <div>
            <h2 className="text-xl font-semibold text-gray-800 mb-1">
                Monthly Heatmap
            </h2>
            <p className="text-sm text-gray-500 mb-4">
                Comics published per month, displayed as a heatmap. Darker cells
                indicate more comics. Each row is a year; each column is a
                month.
            </p>
            <div className="overflow-x-auto">
                <table
                    className="text-xs border-separate"
                    style={{ borderSpacing: '2px' }}
                >
                    <thead>
                        <tr>
                            <th className="pr-3 text-right text-gray-400 font-medium w-14" />
                            {MONTH_LABELS.map((m) => (
                                <th
                                    key={m}
                                    className="w-10 text-center text-gray-400 font-medium pb-1"
                                >
                                    {m}
                                </th>
                            ))}
                        </tr>
                    </thead>
                    <tbody>
                        {years.map((year) => {
                            const counts =
                                grid.get(year) ??
                                (new Array(12).fill(0) as number[]);
                            return (
                                <tr key={year}>
                                    <td className="pr-3 text-right text-gray-500 font-medium">
                                        {year}
                                    </td>
                                    {counts.map((count, mi) => (
                                        <td
                                            key={mi}
                                            title={`${year}-${MONTH_LABELS[mi]}: ${count.toLocaleString()} comics`}
                                            className={`w-10 h-7 text-center rounded ${heatColor(count, maxComics)} ${textColor(count, maxComics)}`}
                                        >
                                            {count > 0 ? count : ''}
                                        </td>
                                    ))}
                                </tr>
                            );
                        })}
                    </tbody>
                </table>
            </div>
            <div className="mt-3 flex items-center gap-2 text-xs text-gray-400">
                <span>Low</span>
                {[
                    'bg-gray-100',
                    'bg-indigo-100',
                    'bg-indigo-200',
                    'bg-indigo-300',
                    'bg-indigo-400',
                    'bg-indigo-600',
                ].map((cls) => (
                    <span
                        key={cls}
                        className={`inline-block w-5 h-4 rounded ${cls}`}
                    />
                ))}
                <span>High</span>
            </div>
        </div>
    );
}
