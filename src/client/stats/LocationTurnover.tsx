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

import type { LocationTurnoverYear } from '../../../bindings/LocationTurnoverYear';

Chart.register(
    BarController,
    BarElement,
    CategoryScale,
    LinearScale,
    Tooltip,
    Legend,
);

interface ChartProps {
    data: LocationTurnoverYear[];
}

function TurnoverChart({ data }: ChartProps) {
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
                        label: 'New',
                        data: data.map((d) => d.newLocations),
                        backgroundColor: 'rgba(34, 197, 94, 0.8)',
                        stack: 'a',
                    },
                    {
                        label: 'Continuing',
                        data: data.map((d) => d.continuingLocations),
                        backgroundColor: 'rgba(16, 185, 129, 0.8)',
                        stack: 'a',
                    },
                    {
                        label: 'Returning',
                        data: data.map((d) => d.returningLocations),
                        backgroundColor: 'rgba(251, 191, 36, 0.8)',
                        stack: 'a',
                    },
                    {
                        label: 'Dropped',
                        data: data.map((d) => d.droppedLocations),
                        backgroundColor: 'rgba(239, 68, 68, 0.8)',
                        stack: 'b',
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
                                `${ctx.dataset.label ?? ''}: ${(ctx.parsed.y as number).toLocaleString()} locations`,
                        },
                    },
                },
                scales: {
                    x: { stacked: true },
                    y: {
                        stacked: true,
                        beginAtZero: true,
                        title: { display: true, text: 'Locations' },
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

export default function LocationTurnover() {
    const [data, setData] = useState<LocationTurnoverYear[] | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/location-turnover')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<LocationTurnoverYear[]>;
            })
            .then(setData)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    if (error)
        return <p className="text-red-600">Failed to load data: {error}</p>;

    return (
        <div>
            <h2 className="text-xl font-semibold text-gray-800 mb-1">
                Location Turnover
            </h2>
            <p className="text-sm text-gray-500 mb-4">
                Year-by-year breakdown of location composition: new locations
                (first appearance ever), continuing locations (also appeared the
                prior year), returning locations (absent at least one year), and
                locations dropped (appeared last year but not this one).
                &ldquo;Dropped&rdquo; is shown on a separate stack for clarity.
            </p>
            {!data ? (
                <p className="text-gray-500">Loading…</p>
            ) : (
                <TurnoverChart data={data} />
            )}
        </div>
    );
}
