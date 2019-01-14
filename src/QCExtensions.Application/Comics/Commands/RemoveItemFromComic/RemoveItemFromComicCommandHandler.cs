using System.Threading;
using System.Threading.Tasks;
using MediatR;
using QCExtensions.Application.Extensions.DbContext;
using QCExtensions.Application.Interfaces;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Application.Comics.Commands.RemoveItemFromComic
{
	public class RemoveItemFromComicCommandHandler : IRequestHandler<RemoveItemFromComicCommand, Unit>
	{
		private readonly DomainDbContext _context;
		private readonly IActionLogger _actionLogger;

		public RemoveItemFromComicCommandHandler(
			DomainDbContext context,
			IActionLogger actionLogger
			)
		{
			_context = context;
			_actionLogger = actionLogger;
		}

		public async Task<Unit> Handle(RemoveItemFromComicCommand request, CancellationToken cancellationToken)
		{
			if (!await _context.Comics.ExistsAsync(request.ComicId))
			{
				throw new RemoveItemFromComicException("Comic does not exist");
			}
			using (var transaction = _context.Database.BeginTransaction())
			{
				var item = await _context.Items.GetByIdAsync(request.ItemId);
				if (item == null)
				{
					throw new RemoveItemFromComicException("Item does not exist");
				}

				var occurrence = await _context.Occurrences.GetByComicIdAndItemIdAsync(request.ComicId, item.Id);
				if (occurrence == null)
				{
					throw new RemoveItemFromComicException("Item is not in comic");
				}

				_context.Occurrences.Remove(occurrence);
				await _context.SaveChangesAsync(cancellationToken);
				await _actionLogger.LogAsync(request.Token.Value, $"Removed {item.Type} #{item.Id} ({item.Name}) from comic #{request.ComicId}", cancellationToken: cancellationToken);

				transaction.Commit();
			}
			return Unit.Value;
		}
	}
}
