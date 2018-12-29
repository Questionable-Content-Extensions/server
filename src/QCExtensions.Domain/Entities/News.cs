using System;
using System.Collections.Generic;

namespace QCExtensions.Domain.Entities
{
	public class News
	{
		public int ComicId { get; set; }
		public DateTime LastUpdated { get; set; }
		public string NewsText { get; set; }
		public double UpdateFactor { get; set; }
		public bool IsLocked { get; set; }
		public bool IsOutdated
		{
			get
			{
				return !IsLocked
					&& UpdateFactor < 12
					&& (DateTime.UtcNow - LastUpdated).TotalDays > 31 * UpdateFactor;
			}
		}

		public Comic Comic { get; set; }
	}
}