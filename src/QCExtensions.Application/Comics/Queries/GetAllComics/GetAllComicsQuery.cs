using MediatR;
using QCExtensions.Application.Comics.Models;
using System.Collections.Generic;

namespace QCExtensions.Application.Comics.Queries.GetAllComics
{
	public class GetAllComicsQuery : IRequest<List<ComicListDto>>
	{
	}
}
