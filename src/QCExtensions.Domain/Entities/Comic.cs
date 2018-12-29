using System;
using System.Collections.Generic;

namespace QCExtensions.Domain.Entities
{
	public class Comic
	{
		public Comic()
		{
			Occurrences = new HashSet<Occurrence>();
		}
		public int Id { get; set; }
		public bool IsGuestComic { get; set; }
		public bool IsNonCanon { get; set; }
		public bool HasNoCast { get; set; }
		public bool HasNoLocation { get; set; }
		public bool HasNoStoryline { get; set; }
		public bool HasNoTitle { get; set; }
		public bool HasNoTagline { get; set; }
		public string Title { get; set; }
		public string Tagline { get; set; }
		public DateTime? PublishDate { get; set; }
		public bool IsAccuratePublishDate { get; set; }
		public News News { get; set; }
		public ICollection<Occurrence> Occurrences { get; private set; }
	}
}
