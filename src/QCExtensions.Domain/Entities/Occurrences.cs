using System;
using System.Collections.Generic;

namespace QCExtensions.Domain.Entities
{
	public class Occurrence
	{
		public int ComicId { get; set; }
		public Comic Comic { get; set; }
		public int ItemId { get; set; }
		public Item Item { get; set; }
	}
}