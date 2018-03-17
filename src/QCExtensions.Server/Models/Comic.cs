using System;
using System.Collections.Generic;
using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;
using Microsoft.EntityFrameworkCore;

namespace QCExtensions.Server.Models
{
	[Table("comic")]
	public class Comic
	{
		[Key]
		[Column("id")]
		public int Id { get; set; }

		[Column("isGuestComic")]
		public bool IsGuestComic { get; set; }

		[Column("isNonCanon")]
		public bool IsNonCanon { get; set; }

		[Column("title")]
		public string Title { get; set; }

		[Column("tagline")]
		public string Tagline { get; set; }

		[Column("publishDate")]
		public DateTime? PublishDate { get; set; }

		[Column("isAccuratePublishDate")]
		public bool IsAccuratePublishDate { get; set; }

		[ForeignKey("Id")]
		public News News { get; set; }

		public ICollection<Occurrences> Occurrences { get; set; }
	}
}