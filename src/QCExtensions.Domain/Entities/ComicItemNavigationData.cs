namespace QCExtensions.Domain.Entities
{
	public class ComicItemNavigationData
	{
		public int Id { get; set; }
		public int? First { get; set; }
		public int? Previous { get; set; }
		public int? Next { get; set; }
		public int? Last { get; set; }
		public int? Count { get; set; }
	}
}