using System;
using Newtonsoft.Json;

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
		public int? Previous { get; set; }
		public int? Next { get; set; }

		[JsonProperty(NullValueHandling = NullValueHandling.Ignore)]
		public EditorData EditorData { get; set; }
		public ItemWithNavigationData[] Items { get; set; }
		[JsonProperty(NullValueHandling = NullValueHandling.Ignore)]
		public ItemWithNavigationData[] AllItems { get; set; }
	}

	public class EditorData
	{
		public MissingNavigationData Missing { get; set; }
	}

	public class MissingNavigationData
	{
		public NavigationData Cast { get; set; }
		public NavigationData Location { get; set; }
		public NavigationData Storyline { get; set; }
		public NavigationData Title { get; set; }
		public NavigationData Tagline { get; set; }
	}

	public class NavigationData
	{
		public int? First { get; set; }
		public int? Previous { get; set; }
		public int? Next { get; set; }
		public int? Last { get; set; }
	}
}