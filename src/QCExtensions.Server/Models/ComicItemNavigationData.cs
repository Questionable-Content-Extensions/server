using System;
using System.Collections.Generic;
using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;
using Microsoft.EntityFrameworkCore;

namespace QCExtensions.Server.Models
{
	public class ComicItemNavigationData
	{
		public int Id { get; set; }
		public int? First { get; set; }
		public int? Previous { get; set; }
		public int? Next { get; set; }
		public int? Last { get; set; }
		public int? Count { get; set; }
	}
}