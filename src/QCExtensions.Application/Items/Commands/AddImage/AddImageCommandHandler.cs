using System.Threading;
using System.Threading.Tasks;
using Force.Crc32;
using MediatR;
using QCExtensions.Application.Extensions.DbContext;
using QCExtensions.Application.Interfaces;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Application.Items.Commands.AddImage
{
	public class AddImageCommandHandler : IRequestHandler<AddImageCommand, Unit>
	{
		private readonly DomainDbContext _context;
		private readonly IActionLogger _actionLogger;

		public AddImageCommandHandler(
			DomainDbContext context,
			IActionLogger actionLogger
			)
		{
			_context = context;
			_actionLogger = actionLogger;
		}
		
		public async Task<Unit> Handle(AddImageCommand request, CancellationToken cancellationToken)
		{
			if (!await _context.Items.ExistsAsync(request.ItemId))
			{
				throw new Exceptions.ItemDoesNotExistException();
			}

			using (var transaction = _context.Database.BeginTransaction())
			{
				var itemImage = new ItemImage
				{
					ItemId = request.ItemId,
					CRC32CHash = Crc32CAlgorithm.Compute(request.Image),
					Image = request.Image
				};

				_context.ItemImages.Add(itemImage);
				await _context.SaveChangesAsync();
				await _actionLogger.LogAsync(request.Token.Value, $"Uploaded image #{itemImage.Id} for item #{request.ItemId}");

				transaction.Commit();
			}
			return Unit.Value;
		}
	}
}
