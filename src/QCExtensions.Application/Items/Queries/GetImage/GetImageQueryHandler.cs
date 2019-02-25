using AutoMapper;
using MediatR;
using Microsoft.EntityFrameworkCore;
using QCExtensions.Application.Extensions.DbContext;
using QCExtensions.Application.Items.Models;
using QCExtensions.Domain.Entities;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;

namespace QCExtensions.Application.Items.Queries.GetImage
{
	public class GetImageQueryHandler : IRequestHandler<GetImageQuery, byte[]>
	{
		private readonly DomainDbContext _context;
		private readonly IMapper _mapper;

		public GetImageQueryHandler(
			DomainDbContext context,
			IMapper mapper
		)
		{
			_context = context;
			_mapper = mapper;
		}

		public async Task<byte[]> Handle(GetImageQuery request, CancellationToken cancellationToken)
		{
			return await _context.ItemImages.Where(i => i.Id == request.ImageId).Select(i => i.Image).SingleOrDefaultAsync();
		}
	}
}
