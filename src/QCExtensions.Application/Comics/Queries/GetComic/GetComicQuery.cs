using System;
using MediatR;
using QCExtensions.Application.Comics.Models;
using QCExtensions.Application.Interfaces;

namespace QCExtensions.Application.Comics.Queries.GetComic
{
	public class GetComicQuery : RequestWithToken<ComicDto>
	{
		public enum Exclusion
		{
			None,
			Guest,
			NonCanon
		}

		public enum Inclusion
		{
			None,
			All
		}

		public int ComicId { get; set; }
		public Exclusion Exclude { get; set; } = Exclusion.None;
		public Inclusion Include { get; set; } = Inclusion.None;
		
		public override bool AllowInvalidToken => true;
	}
}
