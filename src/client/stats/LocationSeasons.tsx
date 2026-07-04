import {
    BarController,
    BarElement,
    CategoryScale,
    Chart,
    Legend,
    LinearScale,
    Tooltip,
} from 'chart.js';
import type { LocationSeasonEntry } from 'models/LocationSeasonEntry';
import { useEffect, useRef, useState } from 'react';

import { getStatsLocationSeasons } from 'bindings/api/GetStatsLocationSeasons';

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
    entry: LocationSeasonEntry;
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
                        backgroundColor: 'rgba(16, 185, 129, 0.8)',
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

export default function LocationSeasons() {
    const [data, setData] = useState<LocationSeasonEntry[] | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        getStatsLocationSeasons()
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
                Location Seasons
            </h2>
            <p className="text-sm text-gray-500 mb-6">
                Monthly appearance distribution for each location — does a
                location tend to appear more in certain months of the year? Only
                locations with at least 50 appearances are included.
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
