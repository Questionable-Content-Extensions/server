namespace QCExtensions.Domain.Entities
{
	public class ComicEditorData
	{
		public string Type { get; set; }
		public int? First { get; set; }
		public int? Previous { get; set; }
		public int? Next { get; set; }
		public int? Last { get; set; }
	}
}