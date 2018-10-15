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
		[Required]
		public bool IsGuestComic { get; set; }

		[Column("isNonCanon")]
		[Required]
		public bool IsNonCanon { get; set; }

		[Column("title")]
		[MaxLength(255)]
		[Required]
		public string Title { get; set; }

		[Column("tagline")]
		[MaxLength(255)]
		public string Tagline { get; set; }

		[Column("publishDate")]
		public DateTime? PublishDate { get; set; }

		[Column("isAccuratePublishDate")]
		[Required]
		public bool IsAccuratePublishDate { get; set; }

		[ForeignKey("Id")]
		public News News { get; set; }

		public ICollection<Occurrence> Occurrences { get; set; }
	}
}