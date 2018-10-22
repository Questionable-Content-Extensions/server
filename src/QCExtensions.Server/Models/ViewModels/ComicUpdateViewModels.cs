using System;

namespace QCExtensions.Server.Models.ViewModels
{
	public class ComicUpdateViewModelBase
	{
		public Guid Token { get; set; }
		public int Comic { get; set; }
	}

	public class ComicItemAssociationViewModel : ComicUpdateViewModelBase
	{
		public ItemWithTypeViewModel Item { get; set; }
	}

	public class ComicTitleViewModel : ComicUpdateViewModelBase
	{
		public string Title { get; set; }
	}

	public class ComicTaglineViewModel : ComicUpdateViewModelBase
	{
		public string Tagline { get; set; }
	}

	public class ComicPublishDateViewModel : ComicUpdateViewModelBase
	{
		public DateTime PublishDate { get; set; }
		public bool IsAccuratePublishDate { get; set; }
	}

	public class ComicBoolValueViewModel : ComicUpdateViewModelBase
	{
		public bool Value { get; set; }
	}
}
