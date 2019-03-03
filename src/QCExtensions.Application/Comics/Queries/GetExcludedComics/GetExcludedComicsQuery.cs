using MediatR;
using QCExtensions.Application.Comics.Models;
using System.Collections.Generic;

namespace QCExtensions.Application.Comics.Queries.GetExcludedComics
{
	public class GetExcludedComicsQuery : IRequest<List<ComicListDto>>
	{
		public Exclusion ExclusionType { get; set; } = Exclusion.None;
	}
}
