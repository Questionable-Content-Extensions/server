import {
    BarController,
    BarElement,
    CategoryScale,
    Chart,
    LinearScale,
    Tooltip,
} from 'chart.js';
import { useEffect, useRef, useState } from 'react';

import type { PublicationCalendar } from 'bindings/PublicationCalendar';
import { getStatsPublicationCalendar } from 'bindings/api/GetStatsPublicationCalendar';

Chart.register(BarController, BarElement, CategoryScale, LinearScale, Tooltip);

const MONTH_NAMES = [
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

// MySQL DAYOFWEEK: 1=Sunday, 2=Monday, ..., 7=Saturday — ordered Mon-first
const DOW_ORDER = [2, 3, 4, 5, 6, 7, 1];
const DOW_NAMES = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

interface BarProps {
    labels: string[];
    values: number[];
    color: string;
    yLabel: string;
}

function SimpleBarChart({ labels, values, color, yLabel }: BarProps) {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const chartRef = useRef<Chart | null>(null);

    useEffect(() => {
        if (!canvasRef.current) return;
        chartRef.current?.destroy();

        chartRef.current = new Chart(canvasRef.current, {
            type: 'bar',
            data: {
                labels,
                datasets: [
                    {
                        data: values,
                        backgroundColor: color,
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
                                `${(ctx.parsed.y as number).toLocaleString()} comics`,
                        },
                    },
                },
                scales: {
                    x: { grid: { display: false } },
                    y: {
                        beginAtZero: true,
                        title: { display: true, text: yLabel },
                    },
                },
            },
        });

        return () => {
            chartRef.current?.destroy();
            chartRef.current = null;
        };
    }, [labels, values, color, yLabel]);

    return (
        <div style={{ height: '220px' }}>
            <canvas ref={canvasRef} />
        </div>
    );
}

export default function PublicationCalendarPage() {
    const [data, setData] = useState<PublicationCalendar | null>(null);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        getStatsPublicationCalendar()
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

    const allMonths = Array.from({ length: 12 }, (_, i) => i + 1);
    const monthCounts = allMonths.map(
        (m) => data.monthly.find((r) => r.month === m)?.comics ?? 0,
    );

    const dowCounts = DOW_ORDER.map(
        (d) => data.daily.find((r) => r.dow === d)?.comics ?? 0,
    );

    return (
        <div>
            <h2 className="text-xl font-semibold text-gray-800 mb-1">
                Publication Calendar
            </h2>
            <p className="text-sm text-gray-500 mb-6">
                How comics are distributed across months and days of the week.
            </p>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                <div>
                    <h3 className="text-base font-medium text-gray-700 mb-3">
                        Comics by month
                    </h3>
                    <SimpleBarChart
                        labels={MONTH_NAMES}
                        values={monthCounts}
                        color="rgba(99, 102, 241, 0.7)"
                        yLabel="Comics published"
                    />
                </div>
                <div>
                    <h3 className="text-base font-medium text-gray-700 mb-3">
                        Comics by day of week
                    </h3>
                    <SimpleBarChart
                        labels={DOW_NAMES}
                        values={dowCounts}
                        color="rgba(251, 146, 60, 0.7)"
                        yLabel="Comics published"
                    />
                </div>
            </div>
        </div>
    );
}
