import type { ItemStats } from 'models/ItemStats';

import ItemStatsTable from './ItemStatsTable';

interface CharacterRankingsProps {
    sharedData?: ItemStats[] | null;
    sharedError?: string | null;
}

export default function CharacterRankings({
    sharedData,
    sharedError,
}: CharacterRankingsProps) {
    return (
        <ItemStatsTable
            endpoint="/api/v3/stats/cast"
            title="Character Rankings"
            description="All cast members ranked by total number of comic appearances."
            sortBy="appearances"
            sharedData={sharedData}
            sharedError={sharedError}
        />
    );
}
