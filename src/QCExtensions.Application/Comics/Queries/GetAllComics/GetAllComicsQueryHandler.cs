using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using AutoMapper;
using MediatR;
using Microsoft.EntityFrameworkCore;
using QCExtensions.Application.Comics.Models;
using QCExtensions.Application.Exceptions;
using QCExtensions.Application.Extensions.DbContext;
using QCExtensions.Application.Interfaces;
using QCExtensions.Application.Items.Models;
using QCExtensions.Domain.Entities;
using static QCExtensions.Application.Comics.Queries.GetComic.GetComicQuery;

namespace QCExtensions.Application.Comics.Queries.GetAllComics.GetComic
{
	public class GetAllComicsQueryHandler : IRequestHandler<GetAllComicsQuery, List<ComicListDto>>
	{
		private readonly DomainDbContext _context;

		public GetAllComicsQueryHandler(DomainDbContext context)
		{
			_context = context;
		}

		public async Task<List<ComicListDto>> Handle(GetAllComicsQuery request, CancellationToken cancellationToken)
		{
			var comicList = await _context.Comics.Select(c => new ComicListDto { Comic = c.Id, Title = c.Title }).ToListAsync();
			return comicList;
		}
	}
}
