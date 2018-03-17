namespace QCExtensions.Server.Models.ViewModels
{
	public class ItemViewModel
	{
		public int Id { get; set; }
		public string ShortName { get; set; }
		public string Name { get; set; }
		public string Type { get; set; }
		public string Color { get; set; }

		public int? First { get; set; }
		public int? Last { get; set; }
		public int Appearances { get; set; }
		public int TotalComics { get; set; }
		public double Presence
		{
			get
			{
				return TotalComics == 0
				? 0
				: Appearances * 100.0 / TotalComics;
			}
		}
		public bool HasImage { get; set; }
	}
}
