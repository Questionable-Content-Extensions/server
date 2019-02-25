using AutoMapper;
using MediatR;
using Microsoft.EntityFrameworkCore;
using QCExtensions.Application.Extensions.DbContext;
using QCExtensions.Application.Items.Models;
using QCExtensions.Domain.Entities;
using QCExtensions.Domain.Enumerations;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;

namespace QCExtensions.Application.Items.Queries.GetRelatedItems
{
	public class GetRelatedItemsQueryHandler : IRequestHandler<GetRelatedItemsQuery, List<ItemListDto>>
	{
		private readonly DomainDbContext _context;
		private readonly IMapper _mapper;

		public GetRelatedItemsQueryHandler(
			DomainDbContext context,
			IMapper mapper
		)
		{
			_context = context;
			_mapper = mapper;
		}

		public async Task<List<ItemListDto>> Handle(GetRelatedItemsQuery request, CancellationToken cancellationToken)
		{
			var id = request.ItemId;
			var type = request.Type.ToStringRepresentation();
			var amount = request.Amount;

			if (!await _context.Items.ExistsAsync(id))
			{
				return null;
			}

			var typeWithQuery = (from i in _context.Items
								 from o in i.Occurrences
								 join o2 in _context.Occurrences on o.ComicId equals o2.ComicId
								 let i2 = o2.Item
								 where i.Id == id && i2.Id != i.Id && i2.Type == type
								 group i2 by i2.Id into gi2
								 orderby gi2.Count() descending
								 select gi2)
								 .Take(amount)
								 .Select(gi2 => new
								 {
									 gi2.First().Id,
									 gi2.First().ShortName,
									 gi2.First().Name,
									 gi2.First().Type,
									 gi2.First().Color,
									 Count = gi2.Count()
								 });

			return _mapper.Map<List<ItemListDto>>(await typeWithQuery.ToListAsync());
		}
	}
}
