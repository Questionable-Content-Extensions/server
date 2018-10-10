namespace QCExtensions.Server.Models.ViewModels
{
	public abstract class ItemViewModelBase
	{
		public int Id { get; set; }
		public string ShortName { get; set; }
		public string Name { get; set; }
		public string Type { get; set; }
		public string Color { get; set; }
	}

	public class ItemViewModel : ItemViewModelBase
	{
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

	public class ItemWithTypeViewModel : ItemViewModelBase
	{
		public int Count { get; set; }
	}
}
