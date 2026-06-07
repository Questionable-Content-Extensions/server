import { useEffect, useState } from 'react';
import { NavLink, Route, Routes } from 'react-router-dom';

import type { ItemStats } from '../../../bindings/ItemStats';
import BestFriendScore from './BestFriendScore';
import CharacterDebuts from './CharacterDebuts';
import CharacterLongevity from './CharacterLongevity';
import CharacterRankings from './CharacterRankings';
import CharacterRegularity from './CharacterRegularity';
import CoAppearances from './CoAppearances';
import ComebackCharacters from './ComebackCharacters';
import DebutClusters from './DebutClusters';
import DebutsPerYear from './DebutsPerYear';
import EnsembleRatio from './EnsembleRatio';
import LocationAffinity from './LocationAffinity';
import LocationCoOccurrences from './LocationCoOccurrences';
import LocationDebuts from './LocationDebuts';
import LocationStats from './LocationStats';
import LocationYearlySpotlight from './LocationYearlySpotlight';
import MostCrowdedComics from './MostCrowdedComics';
import OneHitWonders from './OneHitWonders';
import PublicationCalendar from './PublicationCalendar';
import PublicationGaps from './PublicationGaps';
import RetiredCharacters from './RetiredCharacters';
import YearlyOverview from './YearlyOverview';
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

interface NavRowProps {
    label: string;
    children: React.ReactNode;
}

function NavRow({ label, children }: NavRowProps) {
    return (
        <div className="flex items-center gap-1 border-b border-gray-200 last:border-b-0">
            <span className="w-32 shrink-0 text-xs font-semibold text-gray-400 uppercase tracking-wide px-2">
                {label}
            </span>
            <div className="flex gap-1 flex-wrap">{children}</div>
        </div>
    );
}

function StatsNav() {
    return (
        <nav className="mb-6 border border-gray-200 rounded-md divide-y divide-gray-200">
            <NavRow label="Characters">
                <TabLink to="/stats">Rankings</TabLink>
                <TabLink to="/stats/debuts">Debuts</TabLink>
                <TabLink to="/stats/longevity">Longevity</TabLink>
                <TabLink to="/stats/one-hit-wonders">One-Hit Wonders</TabLink>
                <TabLink to="/stats/comeback-characters">
                    Comeback Characters
                </TabLink>
                <TabLink to="/stats/retired-characters">
                    Retired Characters
                </TabLink>
                <TabLink to="/stats/character-regularity">Regularity</TabLink>
            </NavRow>
            <NavRow label="Locations">
                <TabLink to="/stats/locations">Location Stats</TabLink>
                <TabLink to="/stats/location-debuts">Debuts</TabLink>
                <TabLink to="/stats/location-yearly-spotlight">
                    Spotlight
                </TabLink>
                <TabLink to="/stats/location-co-occurrences">
                    Co-Occurrences
                </TabLink>
                <TabLink to="/stats/location-affinity">Affinity</TabLink>
            </NavRow>
            <NavRow label="Relationships">
                <TabLink to="/stats/co-appearances">
                    Who Appears Together
                </TabLink>
                <TabLink to="/stats/best-friend-score">
                    Best Friend Score
                </TabLink>
                <TabLink to="/stats/ensemble-ratio">Ensemble Ratio</TabLink>
            </NavRow>
            <NavRow label="Publication & Time">
                <TabLink to="/stats/yearly-spotlight">Yearly Spotlight</TabLink>
                <TabLink to="/stats/yearly-overview">Yearly Overview</TabLink>
                <TabLink to="/stats/debuts-per-year">Debuts Per Year</TabLink>
                <TabLink to="/stats/debut-clusters">Debut Clusters</TabLink>
                <TabLink to="/stats/publication-calendar">
                    Publication Calendar
                </TabLink>
                {/* <TabLink to="/stats/publication-gaps">Publication Gaps</TabLink> */}
                <TabLink to="/stats/crowded-comics">Crowded Comics</TabLink>
            </NavRow>
        </nav>
    );
}

export default function StatsLayout() {
    const [castData, setCastData] = useState<ItemStats[] | null>(null);
    const [castError, setCastError] = useState<string | null>(null);
    const [locationsData, setLocationsData] = useState<ItemStats[] | null>(
        null,
    );
    const [locationsError, setLocationsError] = useState<string | null>(null);

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

    useEffect(() => {
        fetch('/api/v3/stats/locations')
            .then((r) => {
                if (!r.ok) throw new Error(`HTTP ${r.status}`);
                return r.json() as Promise<ItemStats[]>;
            })
            .then(setLocationsData)
            .catch((e: unknown) =>
                setLocationsError(e instanceof Error ? e.message : String(e)),
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
                <Route
                    path="locations"
                    element={
                        <LocationStats
                            sharedData={locationsData}
                            sharedError={locationsError}
                        />
                    }
                />
                <Route
                    path="debuts"
                    element={
                        <CharacterDebuts
                            sharedData={castData}
                            sharedError={castError}
                        />
                    }
                />
                <Route
                    path="location-debuts"
                    element={
                        <LocationDebuts
                            sharedData={locationsData}
                            sharedError={locationsError}
                        />
                    }
                />
                <Route
                    path="longevity"
                    element={
                        <CharacterLongevity
                            sharedData={castData}
                            sharedError={castError}
                        />
                    }
                />
                <Route
                    path="one-hit-wonders"
                    element={
                        <OneHitWonders
                            castData={castData}
                            castError={castError}
                            locationsData={locationsData}
                            locationsError={locationsError}
                        />
                    }
                />
                <Route path="co-appearances" element={<CoAppearances />} />
                <Route
                    path="location-affinity"
                    element={<LocationAffinity />}
                />
                <Route path="yearly-spotlight" element={<YearlySpotlight />} />
                <Route path="yearly-overview" element={<YearlyOverview />} />
                <Route path="debuts-per-year" element={<DebutsPerYear />} />
                <Route
                    path="publication-calendar"
                    element={<PublicationCalendar />}
                />
                <Route
                    path="comeback-characters"
                    element={<ComebackCharacters />}
                />
                <Route path="crowded-comics" element={<MostCrowdedComics />} />
                <Route
                    path="retired-characters"
                    element={
                        <RetiredCharacters
                            sharedData={castData}
                            sharedError={castError}
                        />
                    }
                />
                <Route
                    path="location-yearly-spotlight"
                    element={<LocationYearlySpotlight />}
                />
                <Route path="publication-gaps" element={<PublicationGaps />} />
                <Route path="best-friend-score" element={<BestFriendScore />} />
                <Route path="debut-clusters" element={<DebutClusters />} />
                <Route path="ensemble-ratio" element={<EnsembleRatio />} />
                <Route
                    path="character-regularity"
                    element={<CharacterRegularity />}
                />
                <Route
                    path="location-co-occurrences"
                    element={<LocationCoOccurrences />}
                />
            </Routes>
        </div>
    );
}
