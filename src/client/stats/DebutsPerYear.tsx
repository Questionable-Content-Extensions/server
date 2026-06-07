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

import type { DebutsPerYear } from '../../../bindings/DebutsPerYear';

Chart.register(
    BarController,
    BarElement,
    CategoryScale,
    LinearScale,
    Tooltip,
    Legend,
);

interface Props {
    data: DebutsPerYear[];
}

function DebutsChart({ data }: Props) {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const chartRef = useRef<Chart | null>(null);

    useEffect(() => {
        if (!canvasRef.current) return;
        chartRef.current?.destroy();

        chartRef.current = new Chart(canvasRef.current, {
            type: 'bar',
            data: {
                labels: data.map((d) => String(d.year)),
                datasets: [
                    {
                        label: 'Characters',
                        data: data.map((d) => d.castDebuts),
                        backgroundColor: 'rgba(99, 102, 241, 0.7)',
                    },
                    {
                        label: 'Locations',
                        data: data.map((d) => d.locationDebuts),
                        backgroundColor: 'rgba(251, 146, 60, 0.7)',
                    },
                ],
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                animation: false,
                plugins: {
                    legend: { position: 'bottom' },
                    tooltip: {
                        callbacks: {
                            label: (ctx) =>
                                `${ctx.dataset.label}: ${(ctx.parsed.y as number).toLocaleString()}`,
                        },
                    },
                },
                scales: {
                    x: { stacked: false },
                    y: {
                        beginAtZero: true,
                        title: { display: true, text: 'New debuts' },
                    },
                },
            },
        });

        return () => {
            chartRef.current?.destroy();
            chartRef.current = null;
        };
    }, [data]);

    return (
        <div style={{ height: `${Math.max(300, data.length * 14)}px` }}>
            <canvas ref={canvasRef} />
        </div>
    );
}

export default function DebutsPerYearPage() {
    const [data, setData] = useState<DebutsPerYear[] | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/debuts-per-year')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<DebutsPerYear[]>;
            })
            .then(setData)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    if (error) {
        return <p className="text-red-600">Failed to load data: {error}</p>;
    }

    return (
        <div>
            <h2 className="text-xl font-semibold text-gray-800 mb-1">
                Debuts Per Year
            </h2>
            <p className="text-sm text-gray-500 mb-4">
                How many new characters and locations made their first
                appearance each year.
            </p>
            {!data ? (
                <p className="text-gray-500">Loading…</p>
            ) : (
                <DebutsChart data={data} />
            )}
        </div>
    );
}
