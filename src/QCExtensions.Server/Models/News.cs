using System;
using System.Collections.Generic;
using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;
using Microsoft.EntityFrameworkCore;

namespace QCExtensions.Server.Models
{
	[Table("news")]
	public class News
	{
		[Key]
		[Column("comic")]
		public int Comic { get; set; }

		[Column("lastUpdated")]
		public DateTime LastUpdated { get; set; }

		[Column("news")]
		public string NewsText { get; set; }

		[Column("updateFactor")]
		public double UpdateFactor { get; set; }

		[Column("isLocked")]
		public bool IsLocked { get; set; }
	}
}