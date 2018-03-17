using System;
using System.Collections.Generic;
using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;
using Microsoft.EntityFrameworkCore;

namespace QCExtensions.Server.Models
{
	[Table("occurences")]
	public class Occurrences
	{
		[Column("comic_id")]
		public int ComicId { get; set; }
		public Comic Comic { get; set; }

		[Column("items_id")]
		public int ItemId { get; set; }
		public Item Item { get; set; }
	}
}