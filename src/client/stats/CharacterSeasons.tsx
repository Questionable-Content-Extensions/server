import {
    BarController,
    BarElement,
    CategoryScale,
    Chart,
    Legend,
    LinearScale,
    Tooltip,
} from 'chart.js';
import { useEffect, useRef, useState } from 'react';

import type { CharacterSeasonEntry } from 'bindings/CharacterSeasonEntry';
import { getStatsCharacterSeasons } from 'bindings/api/GetStatsCharacterSeasons';

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
        <div style={{ height: '200px' }}>
            <canvas ref={canvasRef} />
        </div>
    );
}

export default function CharacterSeasons() {
    const [data, setData] = useState<CharacterSeasonEntry[] | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        getStatsCharacterSeasons()
            .then(setData)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    if (error)
        return <p className="text-red-600">Failed to load data: {error}</p>;
    if (!data) return <p className="text-gray-500">Loading…</p>;

    return (
        <div>
            <h2 className="text-xl font-semibold text-gray-800 mb-1">
                Character Seasons
            </h2>
            <p className="text-sm text-gray-500 mb-6">
                Monthly appearance distribution for each character — does a
                character tend to appear more in certain months of the year?
                Only characters with at least 50 appearances are included.
            </p>
            <div className="space-y-8">
                {data.map((entry) => (
                    <div key={entry.id}>
                        <h3 className="text-sm font-semibold text-gray-700 mb-1">
                            {entry.name}
                        </h3>
                        <SeasonChart entry={entry} />
                    </div>
                ))}
            </div>
        </div>
    );
}
