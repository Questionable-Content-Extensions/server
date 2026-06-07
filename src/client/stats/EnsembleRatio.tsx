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

import type { EnsembleRatio } from '../../../bindings/EnsembleRatio';

Chart.register(
    BarController,
    BarElement,
    CategoryScale,
    LinearScale,
    Tooltip,
    Legend,
);

interface ChartProps {
    data: EnsembleRatio[];
}

function EnsembleChart({ data }: ChartProps) {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const chartRef = useRef<Chart | null>(null);

    useEffect(() => {
        if (!canvasRef.current) return;
        chartRef.current?.destroy();

        const pct = (n: number, total: number) =>
            total > 0 ? (n / total) * 100 : 0;

        chartRef.current = new Chart(canvasRef.current, {
            type: 'bar',
            data: {
                labels: data.map((d) => String(d.year)),
                datasets: [
                    {
                        label: 'No cast',
                        data: data.map((d) => pct(d.noCast, d.total)),
                        backgroundColor: 'rgba(156, 163, 175, 0.8)',
                        stack: 'a',
                    },
                    {
                        label: 'Solo (1)',
                        data: data.map((d) => pct(d.solo, d.total)),
                        backgroundColor: 'rgba(99, 102, 241, 0.8)',
                        stack: 'a',
                    },
                    {
                        label: 'Small group (2–4)',
                        data: data.map((d) => pct(d.smallGroup, d.total)),
                        backgroundColor: 'rgba(251, 146, 60, 0.8)',
                        stack: 'a',
                    },
                    {
                        label: 'Ensemble (5+)',
                        data: data.map((d) => pct(d.largeGroup, d.total)),
                        backgroundColor: 'rgba(34, 197, 94, 0.8)',
                        stack: 'a',
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
                            label: (ctx) => {
                                const row = data[ctx.dataIndex];
                                const label = ctx.dataset.label ?? '';
                                const raw = ctx.parsed.y as number;
                                let count = 0;
                                if (label === 'No cast') count = row.noCast;
                                else if (label === 'Solo (1)') count = row.solo;
                                else if (label === 'Small group (2–4)')
                                    count = row.smallGroup;
                                else if (label === 'Ensemble (5+)')
                                    count = row.largeGroup;
                                return `${label}: ${raw.toFixed(1)}% (${count.toLocaleString()} comics)`;
                            },
                        },
                    },
                },
                scales: {
                    x: { stacked: true },
                    y: {
                        stacked: true,
                        beginAtZero: true,
                        max: 100,
                        title: {
                            display: true,
                            text: '% of comics',
                        },
                        ticks: {
                            callback: (val) => `${String(val)}%`,
                        },
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
        <div style={{ height: '400px' }}>
            <canvas ref={canvasRef} />
        </div>
    );
}

export default function EnsembleRatioPage() {
    const [data, setData] = useState<EnsembleRatio[] | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/ensemble-ratio')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<EnsembleRatio[]>;
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
                Ensemble vs. Solo Ratio
            </h2>
            <p className="text-sm text-gray-500 mb-4">
                Each year broken down by how many cast members are tagged per
                comic: no cast, solo, small group (2–4), or ensemble (5+). Shows
                whether the comic has become more of an ensemble over time.
            </p>
            {!data ? (
                <p className="text-gray-500">Loading…</p>
            ) : (
                <EnsembleChart data={data} />
            )}
        </div>
    );
}
