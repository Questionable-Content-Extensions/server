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

namespace QCExtensions.Application.Comics.Queries.GetExcludedComics
{
	public class GetExcludedComicsQueryHandler : IRequestHandler<GetExcludedComicsQuery, List<ComicListDto>>
	{
		private readonly DomainDbContext _context;

		public GetExcludedComicsQueryHandler(DomainDbContext context)
		{
			_context = context;
		}

		public async Task<List<ComicListDto>> Handle(GetExcludedComicsQuery request, CancellationToken cancellationToken)
		{
			IQueryable<Comic> comicListQuery =  _context.Comics;
			
			if (request.ExclusionType == Exclusion.Guest)
			{
				comicListQuery = comicListQuery.Where(c => c.IsGuestComic);
			}	
			else if (request.ExclusionType == Exclusion.NonCanon)
			{
				comicListQuery = comicListQuery.Where(c => c.IsNonCanon);
			}
			
			return await comicListQuery.Select(c => new ComicListDto
			{
				Comic = c.Id,
				Title = c.Title,
				IsGuestComic = c.IsGuestComic,
				IsNonCanon = c.IsNonCanon
			}).ToListAsync();
		}
	}
}
