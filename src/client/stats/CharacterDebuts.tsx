import type { ItemStats } from '../../../bindings/ItemStats';
import ItemStatsTable from './ItemStatsTable';

interface CharacterDebutsProps {
    sharedData?: ItemStats[] | null;
    sharedError?: string | null;
}

export default function CharacterDebuts({
    sharedData,
    sharedError,
}: CharacterDebutsProps) {
    return (
        <ItemStatsTable
            endpoint="/api/v3/stats/cast"
            title="Character Debuts"
            description="All cast members in order of first appearance, showing when they joined the comic."
            sortBy="firstComic"
            sharedData={sharedData}
            sharedError={sharedError}
        />
    );
}
