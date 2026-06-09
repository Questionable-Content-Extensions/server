import { useEffect, useState } from 'react';
import { NavLink, Route, Routes } from 'react-router-dom';

import type { ItemStats } from 'bindings/ItemStats';
import { getStatsCast } from 'bindings/api/GetStatsCast';
import { getStatsLocations } from 'bindings/api/GetStatsLocations';

import AppearanceDistribution from './AppearanceDistribution';
import BestFriendScore from './BestFriendScore';
import BreakoutYears from './BreakoutYears';
import CastTurnover from './CastTurnover';
import CharacterDebuts from './CharacterDebuts';
import CharacterHomeTurf from './CharacterHomeTurf';
import CharacterLongevity from './CharacterLongevity';
import CharacterRankings from './CharacterRankings';
import CharacterRegularity from './CharacterRegularity';
import CharacterSeasons from './CharacterSeasons';
import CoAppearances from './CoAppearances';
import ComebackCharacters from './ComebackCharacters';
import ComebackLocations from './ComebackLocations';
import DebutYears from './DebutYears';
import DebutsPerYear from './DebutsPerYear';
import EnsembleRatio from './EnsembleRatio';
import LocationAffinity from './LocationAffinity';
import LocationAppearanceDistribution from './LocationAppearanceDistribution';
import LocationBreakoutYears from './LocationBreakoutYears';
import LocationCoOccurrences from './LocationCoOccurrences';
import LocationDebuts from './LocationDebuts';
import LocationLifespan from './LocationLifespan';
import LocationOneHitWonders from './LocationOneHitWonders';
import LocationRegularity from './LocationRegularity';
import LocationSeasons from './LocationSeasons';
import LocationSocialHub from './LocationSocialHub';
import LocationStats from './LocationStats';
import LocationTurnover from './LocationTurnover';
import LocationYearlySpotlight from './LocationYearlySpotlight';
import LonerIndex from './LonerIndex';
import MilestoneTracker from './MilestoneTracker';
import MonthlyHeatmap from './MonthlyHeatmap';
import MostCrowdedComics from './MostCrowdedComics';
import NeverMet from './NeverMet';
import OneHitWonders from './OneHitWonders';
import PairEvolution from './PairEvolution';
import PublicationCalendar from './PublicationCalendar';
import PublicationGaps from './PublicationGaps';
import PublicationStreaks from './PublicationStreaks';
import RetiredCharacters from './RetiredCharacters';
import RetiredLocations from './RetiredLocations';
import ScheduleEvolution from './ScheduleEvolution';
import SocialHub from './SocialHub';
import TrendingCharacters from './TrendingCharacters';
import TrendingLocations from './TrendingLocations';
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
                <TabLink to="/stats/social-hub">Social Hub</TabLink>
                <TabLink to="/stats/trending-characters">Trending</TabLink>
                <TabLink to="/stats/cast-turnover">Turnover</TabLink>
                <TabLink to="/stats/character-seasons">Seasons</TabLink>
                <TabLink to="/stats/breakout-years">Breakout Years</TabLink>
                <TabLink to="/stats/appearance-distribution">
                    Appearance Distribution
                </TabLink>
                <TabLink to="/stats/yearly-spotlight">Yearly Spotlight</TabLink>
            </NavRow>
            <NavRow label="Locations">
                <TabLink to="/stats/locations">Rankings</TabLink>
                <TabLink to="/stats/location-debuts">Debuts</TabLink>
                <TabLink to="/stats/location-lifespan">Lifespan</TabLink>
                <TabLink to="/stats/location-one-hit-wonders">
                    One-Hit Wonders
                </TabLink>
                <TabLink to="/stats/comeback-locations">
                    Comeback Locations
                </TabLink>
                <TabLink to="/stats/retired-locations">
                    Retired Locations
                </TabLink>
                <TabLink to="/stats/location-regularity">Regularity</TabLink>
                <TabLink to="/stats/location-social-hub">Social Hub</TabLink>
                <TabLink to="/stats/trending-locations">Trending</TabLink>
                <TabLink to="/stats/location-turnover">Turnover</TabLink>
                <TabLink to="/stats/location-seasons">Seasons</TabLink>
                <TabLink to="/stats/location-breakout-years">
                    Breakout Years
                </TabLink>
                <TabLink to="/stats/location-appearance-distribution">
                    Appearance Distribution
                </TabLink>
                <TabLink to="/stats/location-yearly-spotlight">
                    Yearly Spotlight
                </TabLink>
                <TabLink to="/stats/location-co-occurrences">
                    Co-Occurrences
                </TabLink>
                <TabLink to="/stats/location-affinity">Affinity</TabLink>
            </NavRow>
            <NavRow label="Relationships">
                <TabLink to="/stats/co-appearances">Co-Appearances</TabLink>
                <TabLink to="/stats/best-friend-score">
                    Best Friend Score
                </TabLink>
                <TabLink to="/stats/character-home-turf">Home Turf</TabLink>
                <TabLink to="/stats/pair-evolution">Pair Evolution</TabLink>
                <TabLink to="/stats/loner-index">Loner Index</TabLink>
                <TabLink to="/stats/never-met">Never Met</TabLink>
            </NavRow>
            <NavRow label="Publication & Time">
                <TabLink to="/stats/debuts-per-year">Debuts Per Year</TabLink>
                <TabLink to="/stats/debut-years">Debut Years</TabLink>
                <TabLink to="/stats/publication-calendar">
                    Publication Calendar
                </TabLink>
                {/* <TabLink to="/stats/publication-gaps">Publication Gaps</TabLink> */}
                <TabLink to="/stats/crowded-comics">Crowded Comics</TabLink>
                <TabLink to="/stats/ensemble-ratio">Ensemble Ratio</TabLink>
                <TabLink to="/stats/schedule-evolution">
                    Schedule Evolution
                </TabLink>
                <TabLink to="/stats/publication-streaks">
                    Publication Streaks
                </TabLink>
                <TabLink to="/stats/monthly-heatmap">Monthly Heatmap</TabLink>
                <TabLink to="/stats/milestones">Milestones</TabLink>
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
        getStatsCast()
            .then(setCastData)
            .catch((e: unknown) =>
                setCastError(e instanceof Error ? e.message : String(e)),
            );
    }, []);

    useEffect(() => {
        getStatsLocations()
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
                        />
                    }
                />
                <Route
                    path="location-one-hit-wonders"
                    element={
                        <LocationOneHitWonders
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
                <Route path="debuts-per-year" element={<DebutsPerYear />} />
                <Route
                    path="publication-calendar"
                    element={<PublicationCalendar />}
                />
                <Route
                    path="comeback-characters"
                    element={<ComebackCharacters />}
                />
                <Route
                    path="comeback-locations"
                    element={<ComebackLocations />}
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
                    path="retired-locations"
                    element={
                        <RetiredLocations
                            sharedData={locationsData}
                            sharedError={locationsError}
                        />
                    }
                />
                <Route
                    path="location-yearly-spotlight"
                    element={<LocationYearlySpotlight />}
                />
                <Route path="publication-gaps" element={<PublicationGaps />} />
                <Route path="best-friend-score" element={<BestFriendScore />} />
                <Route path="debut-years" element={<DebutYears />} />
                <Route path="ensemble-ratio" element={<EnsembleRatio />} />
                <Route
                    path="character-regularity"
                    element={<CharacterRegularity />}
                />
                <Route
                    path="location-regularity"
                    element={<LocationRegularity />}
                />
                <Route
                    path="location-co-occurrences"
                    element={<LocationCoOccurrences />}
                />
                <Route path="social-hub" element={<SocialHub />} />
                <Route
                    path="trending-characters"
                    element={<TrendingCharacters />}
                />
                <Route path="cast-turnover" element={<CastTurnover />} />
                <Route
                    path="character-seasons"
                    element={<CharacterSeasons />}
                />
                <Route path="location-seasons" element={<LocationSeasons />} />
                <Route path="breakout-years" element={<BreakoutYears />} />
                <Route
                    path="location-breakout-years"
                    element={<LocationBreakoutYears />}
                />
                <Route
                    path="appearance-distribution"
                    element={
                        <AppearanceDistribution
                            castData={castData}
                            castError={castError}
                        />
                    }
                />
                <Route
                    path="location-appearance-distribution"
                    element={
                        <LocationAppearanceDistribution
                            locationsData={locationsData}
                            locationsError={locationsError}
                        />
                    }
                />
                <Route
                    path="location-social-hub"
                    element={<LocationSocialHub />}
                />
                <Route
                    path="location-turnover"
                    element={<LocationTurnover />}
                />
                <Route
                    path="location-lifespan"
                    element={
                        <LocationLifespan
                            locationsData={locationsData}
                            locationsError={locationsError}
                        />
                    }
                />
                <Route
                    path="trending-locations"
                    element={<TrendingLocations />}
                />
                <Route
                    path="character-home-turf"
                    element={<CharacterHomeTurf />}
                />
                <Route
                    path="pair-evolution"
                    element={
                        <PairEvolution
                            castData={castData}
                            castError={castError}
                        />
                    }
                />
                <Route path="loner-index" element={<LonerIndex />} />
                <Route path="never-met" element={<NeverMet />} />
                <Route
                    path="schedule-evolution"
                    element={<ScheduleEvolution />}
                />
                <Route
                    path="publication-streaks"
                    element={<PublicationStreaks />}
                />
                <Route path="monthly-heatmap" element={<MonthlyHeatmap />} />
                <Route path="milestones" element={<MilestoneTracker />} />
            </Routes>
        </div>
    );
}
