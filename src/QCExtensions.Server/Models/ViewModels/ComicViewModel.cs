using System;

namespace QCExtensions.Server.Models.ViewModels
{
	public class ComicViewModel
	{
		public int Comic { get; set; }
		public bool HasData { get; set; }
		public DateTime? PublishDate { get; set; }
		public bool IsAccuratePublishDate { get; set; }
		public string Title { get; set; }
		public string Tagline { get; set; }
		public bool IsGuestComic { get; set; }
		public bool IsNonCanon { get; set; }
		public string News { get; set; }

		// Items
		// AllItems

	}
}