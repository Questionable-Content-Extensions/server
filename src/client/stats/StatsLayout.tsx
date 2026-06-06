import { useEffect, useState } from 'react';
import { NavLink, Route, Routes } from 'react-router-dom';

import type { ItemStats } from '../../../bindings/ItemStats';
import CharacterDebuts from './CharacterDebuts';
import CharacterRankings from './CharacterRankings';
import CoAppearances from './CoAppearances';
import LocationStats from './LocationStats';
import YearlySpotlight from './YearlySpotlight';

interface TabLinkProps {
    to: string;
    children: React.ReactNode;
}

function TabLink({ to, children }: TabLinkProps) {
    return (
        <NavLink
            to={to}
            end
            className={({ isActive }) =>
                `px-4 py-2 text-sm font-medium rounded-t-md border-b-2 transition-colors ${
                    isActive
                        ? 'border-blue-500 text-blue-600'
                        : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                }`
            }
        >
            {children}
        </NavLink>
    );
}

function StatsNav() {
    return (
        <nav className="border-b border-gray-200 mb-6">
            <div className="flex gap-1 flex-wrap">
                <TabLink to="/stats">Character Rankings</TabLink>
                <TabLink to="/stats/locations">Location Stats</TabLink>
                <TabLink to="/stats/debuts">Character Debuts</TabLink>
                <TabLink to="/stats/co-appearances">
                    Who Appears Together
                </TabLink>
                <TabLink to="/stats/yearly-spotlight">Yearly Spotlight</TabLink>
            </div>
        </nav>
    );
}

export default function StatsLayout() {
    const [castData, setCastData] = useState<ItemStats[] | null>(null);
    const [castError, setCastError] = useState<string | null>(null);

    useEffect(() => {
        fetch('/api/v3/stats/cast')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<ItemStats[]>;
            })
            .then(setCastData)
            .catch((e: unknown) =>
                setCastError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    return (
        <div className="py-6">
            <h1 className="text-2xl font-bold text-gray-900 mb-4">
                Comic Statistics
            </h1>
            <StatsNav />
            <Routes>
                <Route
                    index
                    element={
                        <CharacterRankings
                            sharedData={castData}
                            sharedError={castError}
                        />
                    }
                />
                <Route path="locations" element={<LocationStats />} />
                <Route
                    path="debuts"
                    element={
                        <CharacterDebuts
                            sharedData={castData}
                            sharedError={castError}
                        />
                    }
                />
                <Route path="co-appearances" element={<CoAppearances />} />
                <Route path="yearly-spotlight" element={<YearlySpotlight />} />
            </Routes>
        </div>
    );
}
