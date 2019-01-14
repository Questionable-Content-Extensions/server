using AutoMapper;
using MediatR;
using Microsoft.EntityFrameworkCore;
using QCExtensions.Application.Comics.Models;
using QCExtensions.Application.Comics.Queries.GetComic;
using QCExtensions.Application.Exceptions;
using QCExtensions.Application.Extensions.DbContext;
using QCExtensions.Application.Interfaces;
using QCExtensions.Application.Items.Models;
using QCExtensions.Domain.Entities;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;

using static QCExtensions.Application.Comics.Queries.GetComic.GetComicQuery;

namespace QCExtensions.Application.Comics.Queries.GetComic
{
	public class GetComicQueryHandler : IRequestHandler<GetComicQuery, ComicDto>
	{
		private readonly DomainDbContext _context;
		private readonly IMapper _mapper;
		private readonly INewsUpdater _newsUpdater;

		public GetComicQueryHandler(
			DomainDbContext context,
			IMapper mapper,
			INewsUpdater newsUpdater)
		{
			_context = context;
			_mapper = mapper;
			_newsUpdater = newsUpdater;
		}

		private async Task<EditorDataDto> GetEditorDataForComicAsync(int comicId)
		{
			var comicEditorData = await _context.QueryComicEditorData(comicId).ToListAsync();

			NavigationDataDto castNavigationData = null, locationNavigationData = null,
				storylineNavigationData = null, titleNavigationData = null, taglineNavigationData = null;
			foreach (var comicEditorValue in comicEditorData)
			{
				switch (comicEditorValue.Type)
				{
					case "cast":
						castNavigationData = _mapper.Map<NavigationDataDto>(comicEditorValue);
						break;

					case "location":
						locationNavigationData = _mapper.Map<NavigationDataDto>(comicEditorValue);
						break;

					case "storyline":
						storylineNavigationData = _mapper.Map<NavigationDataDto>(comicEditorValue);
						break;

					case "title":
						titleNavigationData = _mapper.Map<NavigationDataDto>(comicEditorValue);
						break;

					case "tagline":
						taglineNavigationData = _mapper.Map<NavigationDataDto>(comicEditorValue);
						break;

					default:
						throw new InvalidEditorDataTypeException(comicEditorValue.Type);
				}
			}

			var editorData = new EditorDataDto
			{
				Missing = new MissingNavigationDataDto
				{
					Cast = castNavigationData,
					Location = locationNavigationData,
					Storyline = storylineNavigationData,
					Title = titleNavigationData,
					Tagline = taglineNavigationData
				}
			};

			return editorData;
		}

		private string ExclusionToString(Exclusion exclusion)
		{
			switch (exclusion)
			{
				case Exclusion.Guest:
					return "guest";

				case Exclusion.NonCanon:
					return "non-canon";

				default:
					return null;
			}
		}

		public async Task<ComicDto> Handle(GetComicQuery request, CancellationToken cancellationToken)
		{
			var comic = await _context.Comics.GetByIdAsync(request.ComicId, includeItems: true);
			ComicDto comicDto;
			if (comic != null)
			{
				comicDto = _mapper.Map<ComicDto>(comic);
			}
			else
			{
				comicDto = new ComicDto
				{
					Comic = request.ComicId,
					HasData = false
				};
			}

			var previousQuery = _context.Comics.Where(c => c.Id < comicDto.Comic);
			var nextQuery = _context.Comics.Where(c => c.Id > comicDto.Comic);
			if (request.Exclude == Exclusion.Guest)
			{
				previousQuery = previousQuery.Where(c => !c.IsGuestComic);
				nextQuery = nextQuery.Where(c => !c.IsGuestComic);
			}
			else if (request.Exclude == Exclusion.NonCanon)
			{
				previousQuery = previousQuery.Where(c => !c.IsNonCanon);
				nextQuery = nextQuery.Where(c => !c.IsNonCanon);
			}
			comicDto.Previous = (await previousQuery.OrderByDescending(c => c.Id).Select(c => new { c.Id }).FirstOrDefaultAsync())?.Id;
			comicDto.Next = (await nextQuery.OrderBy(c => c.Id).Select(c => new { c.Id }).FirstOrDefaultAsync())?.Id;

			if (comicDto.HasData)
			{
				_newsUpdater.CheckFor(request.ComicId);
			}

			if (comic != null)
			{
				comicDto.News = (await _context.News.SingleOrDefaultAsync(n => n.ComicId == request.ComicId))?.NewsText;
			}

			if (request.IsValidToken)
			{
				comicDto.EditorData = await GetEditorDataForComicAsync(request.ComicId);
			}

			// UNTESTED CODE:

			var comicItems = comic?.Occurrences.Select(o => o.Item);
			comicDto.Items = comicItems != null
				? _mapper.Map<ItemWithNavigationDataDto[]>(comicItems)
				: new ItemWithNavigationDataDto[0];

			Dictionary<int, ComicItemNavigationData> comicItemNavigationData;
			if (request.Include == Inclusion.All)
			{
				comicItemNavigationData = await _context.QueryComicAllItemNavigationData(request.ComicId, ExclusionToString(request.Exclude)).ToDictionaryAsync(n => n.Id);
			}
			else
			{
				comicItemNavigationData = comicDto.Items.Length > 0
				? await _context.QueryComicItemNavigationData(request.ComicId, ExclusionToString(request.Exclude)).ToDictionaryAsync(n => n.Id)
				: new Dictionary<int, ComicItemNavigationData>();
			}
			foreach (var item in comicDto.Items)
			{
				_mapper.Map(comicItemNavigationData[item.Id], item);
			}
			Array.Sort(comicDto.Items.Select(i => i.Count).ToArray(), comicDto.Items);
			Array.Reverse(comicDto.Items);

			if (request.Include == Inclusion.All)
			{
				var itemsNotInComic = _context.Items.Except(comicItems);
				comicDto.AllItems = _mapper.Map<ItemWithNavigationDataDto[]>(itemsNotInComic);
				foreach (var item in comicDto.AllItems)
				{
					_mapper.Map(comicItemNavigationData[item.Id], item);
				}
				Array.Sort(comicDto.AllItems.Select(i => i.Count).ToArray(), comicDto.AllItems);
				Array.Reverse(comicDto.AllItems);
			}

			return comicDto;
		}
	}
}
