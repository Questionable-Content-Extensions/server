import {
    BarController,
    BarElement,
    CategoryScale,
    Chart,
    Legend,
    LinearScale,
    Tooltip,
} from 'chart.js';
import { useEffect, useMemo, useRef, useState } from 'react';

import type { CharacterSeasonEntry } from '../../../bindings/CharacterSeasonEntry';

Chart.register(
    BarController,
    BarElement,
    CategoryScale,
    LinearScale,
    Tooltip,
    Legend,
);

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

interface ChartProps {
    entry: CharacterSeasonEntry;
}

function SeasonChart({ entry }: ChartProps) {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const chartRef = useRef<Chart | null>(null);

    useEffect(() => {
        if (!canvasRef.current) return;
        chartRef.current?.destroy();

        chartRef.current = new Chart(canvasRef.current, {
            type: 'bar',
            data: {
                labels: MONTH_LABELS,
                datasets: [
                    {
                        label: 'Appearances',
                        data: entry.monthly,
                        backgroundColor: 'rgba(99, 102, 241, 0.8)',
                    },
                ],
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                animation: false,
                plugins: {
                    legend: { display: false },
                    tooltip: {
                        callbacks: {
                            label: (ctx) =>
                                `${(ctx.parsed.y as number).toLocaleString()} appearances`,
                        },
                    },
                },
                scales: {
                    x: {},
                    y: {
                        beginAtZero: true,
                        title: { display: true, text: 'Appearances' },
                    },
                },
            },
        });

        return () => {
            chartRef.current?.destroy();
            chartRef.current = null;
        };
    }, [entry]);

    return (
        <div style={{ height: '300px' }}>
            <canvas ref={canvasRef} />
        </div>
    );
}

export default function CharacterSeasons() {
    const [data, setData] = useState<CharacterSeasonEntry[] | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [selectedId, setSelectedId] = useState<number | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/character-seasons')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<CharacterSeasonEntry[]>;
            })
            .then((d) => {
                setData(d);
                if (d.length > 0) setSelectedId(d[0].id);
            })
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    const selected = useMemo(
        () => data?.find((e) => e.id === selectedId) ?? null,
        [data, selectedId],
    );

    if (error)
        return <p className="text-red-600">Failed to load data: {error}</p>;
    if (!data) return <p className="text-gray-500">Loading…</p>;

    return (
        <div>
            <h2 className="text-xl font-semibold text-gray-800 mb-1">
                Character Seasons
            </h2>
            <p className="text-sm text-gray-500 mb-4">
                Monthly appearance distribution for each character — does a
                character tend to appear more in certain months of the year?
                Only characters with at least 50 appearances are included.
            </p>
            <div className="mb-4">
                <label
                    htmlFor="character-select"
                    className="block text-sm font-medium text-gray-700 mb-1"
                >
                    Character
                </label>
                <select
                    id="character-select"
                    className="border border-gray-300 rounded-md px-3 py-1.5 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                    value={selectedId ?? ''}
                    onChange={(e) => {
                        setSelectedId(Number(e.target.value));
                    }}
                >
                    {data.map((entry) => (
                        <option key={entry.id} value={entry.id}>
                            {entry.name}
                        </option>
                    ))}
                </select>
            </div>
            {selected ? (
                <SeasonChart entry={selected} />
            ) : (
                <p className="text-gray-400">Select a character above.</p>
            )}
        </div>
    );
}
