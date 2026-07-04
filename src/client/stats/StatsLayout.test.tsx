import { describe, expect, it } from 'vitest';

import { getSharedStatsDataRequirements } from './StatsLayout';

describe('getSharedStatsDataRequirements', () => {
    it('requires cast data on cast-consuming routes', () => {
        expect(getSharedStatsDataRequirements('/stats')).toEqual({
            needsCast: true,
            needsLocations: false,
        });
        expect(getSharedStatsDataRequirements('/stats/pair-evolution')).toEqual(
            { needsCast: true, needsLocations: false },
        );
    });

    it('requires location data on location-consuming routes', () => {
        expect(getSharedStatsDataRequirements('/stats/locations')).toEqual({
            needsCast: false,
            needsLocations: true,
        });
        expect(
            getSharedStatsDataRequirements('/stats/location-lifespan'),
        ).toEqual({ needsCast: false, needsLocations: true });
    });

    it('requires neither on routes that fetch their own data', () => {
        expect(getSharedStatsDataRequirements('/stats/co-appearances')).toEqual(
            { needsCast: false, needsLocations: false },
        );
    });
});
