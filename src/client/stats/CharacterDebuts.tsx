import ItemStatsTable from './ItemStatsTable';

export default function CharacterDebuts() {
    return (
        <ItemStatsTable
            endpoint="/api/v3/stats/cast"
            title="Character Debuts"
            description="All cast members in order of first appearance, showing when they joined the comic."
            sortBy="firstComic"
        />
    );
}
