using AutoMapper;
using MediatR;
using Microsoft.EntityFrameworkCore;
using QCExtensions.Application.Items.Models;
using QCExtensions.Domain.Entities;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;

namespace QCExtensions.Application.Items.Queries.GetAllItems
{
	public class GetAllItemsQueryHandler : IRequestHandler<GetAllItemsQuery, List<ItemListDto>>
	{
		private readonly DomainDbContext _context;
		private readonly IMapper _mapper;

		public GetAllItemsQueryHandler(
			DomainDbContext context,
			IMapper mapper
		)
		{
			_context = context;
			_mapper = mapper;
		}

		public async Task<List<ItemListDto>> Handle(GetAllItemsQuery request, CancellationToken cancellationToken)
		{
			var items = await _context.Items.ToArrayAsync();
			var itemDtos = _mapper.Map<ItemListDto[]>(items);

			var comicItemNavigationData = await _context.QueryComicAllItemNavigationData(1).ToDictionaryAsync(n => n.Id);

			foreach (var item in itemDtos)
			{
				_mapper.Map(comicItemNavigationData[item.Id], item);
			}

			Array.Sort(itemDtos.Select(i => i.Count).ToArray(), itemDtos);
			Array.Reverse(itemDtos);

			return itemDtos.ToList();
		}
	}
}
