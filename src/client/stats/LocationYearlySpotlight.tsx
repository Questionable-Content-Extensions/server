import { useEffect, useState } from 'react';

import type { LocationSpotlightResponse } from 'bindings/LocationSpotlightResponse';
import type { YearlySpotlightResponse } from 'bindings/YearlySpotlightResponse';
import { getStatsLocationYearlySpotlight } from 'bindings/api/GetStatsLocationYearlySpotlight';

import YearlyBarChart from './YearlyBarChart';
import YearlyStreamgraph from './YearlyStreamgraph';

type ChartView = 'bar' | 'stream';

function adaptToYearlySpotlight(
    loc: LocationSpotlightResponse,
): YearlySpotlightResponse {
    return {
        characters: loc.locations,
        years: loc.years.map((y) => ({
            year: y.year,
            characters: y.locations,
        })),
    };
}

export default function LocationYearlySpotlight() {
    const [view, setView] = useState<ChartView>('stream');
    const [response, setResponse] = useState<LocationSpotlightResponse | null>(
        null,
    );
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        getStatsLocationYearlySpotlight()
            .then(setResponse)
            .catch((e: unknown) =>
                setError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    if (error) {
        return <p className="text-red-600">Failed to load data: {error}</p>;
    }

    const btnBaseClasses =
        'px-3 py-1.5 text-sm font-medium rounded-md transition-colors';
    const btnActiveClasses = 'bg-indigo-600 text-white';
    const btnInactiveClasses = 'bg-gray-100 text-gray-600 hover:bg-gray-200';

    const adapted = response ? adaptToYearlySpotlight(response) : null;

    return (
        <div>
            <h2 className="text-xl font-semibold text-gray-800 mb-1">
                Location Yearly Spotlight
            </h2>
            <div className="flex items-center gap-4 mb-4">
                <p className="text-sm text-gray-500">
                    {view === 'bar'
                        ? 'Top 5 locations by appearances for each year. Hover a bar for the exact count.'
                        : 'Top 5 locations per year shown as a streamgraph. Hover for details.'}
                </p>
                <div className="ml-auto flex gap-1 shrink-0">
                    <button
                        className={`${btnBaseClasses} ${view === 'bar' ? btnActiveClasses : btnInactiveClasses}`}
                        onClick={() => {
                            setView('bar');
                        }}
                    >
                        Bar chart
                    </button>
                    <button
                        className={`${btnBaseClasses} ${view === 'stream' ? btnActiveClasses : btnInactiveClasses}`}
                        onClick={() => {
                            setView('stream');
                        }}
                    >
                        Streamgraph
                    </button>
                </div>
            </div>
            {!adapted ? (
                <p className="text-gray-500">Loading…</p>
            ) : view === 'bar' ? (
                <YearlyBarChart response={adapted} />
            ) : (
                <YearlyStreamgraph response={adapted} />
            )}
        </div>
    );
}
