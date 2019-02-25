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

namespace QCExtensions.Application.Items.Queries.GetItemImages
{
	public class GetItemImagesQueryHandler : IRequestHandler<GetItemImagesQuery, List<ItemImageDto>>
	{
		private readonly DomainDbContext _context;
		private readonly IMapper _mapper;

		public GetItemImagesQueryHandler(
			DomainDbContext context,
			IMapper mapper
		)
		{
			_context = context;
			_mapper = mapper;
		}

		public async Task<List<ItemImageDto>> Handle(GetItemImagesQuery request, CancellationToken cancellationToken)
		{
			var item = await _context.Items.GetByIdAsync(request.ItemId, includeImages: true);
			if (item == null)
			{
				return null;
			}

			var itemImageDtos = _mapper.Map<List<ItemImageDto>>(item.Images);
			return itemImageDtos;
		}
	}
}
