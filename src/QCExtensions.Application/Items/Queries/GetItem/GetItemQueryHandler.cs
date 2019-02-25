using AutoMapper;
using MediatR;
using Microsoft.EntityFrameworkCore;
using QCExtensions.Application.Extensions.DbContext;
using QCExtensions.Application.Items.Models;
using QCExtensions.Domain.Entities;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;

namespace QCExtensions.Application.Items.Queries.GetItem
{
	public class GetItemQueryHandler : IRequestHandler<GetItemQuery, ItemDto>
	{
		private readonly DomainDbContext _context;
		private readonly IMapper _mapper;

		public GetItemQueryHandler(
			DomainDbContext context,
			IMapper mapper
		)
		{
			_context = context;
			_mapper = mapper;
		}

		public async Task<ItemDto> Handle(GetItemQuery request, CancellationToken cancellationToken)
		{
			var item = await _context.Items.GetByIdAsync(request.ItemId, includeOccurrences: true, includeImages: true);
			if (item == null)
			{
				return null;
			}

			var itemDto = _mapper.Map<ItemDto>(item);

			var appearances = item.Occurrences.Count;

			itemDto.TotalComics = await _context.Comics.CountAsync();
			itemDto.Appearances = item.Occurrences.Count;

			var first = item.Occurrences.Min(o => o.ComicId);
			var last = item.Occurrences.Max(o => o.ComicId);

			itemDto.First = first;
			itemDto.Last = last;

			itemDto.HasImage = item.Images.Any();

			return itemDto;
		}
	}
}
