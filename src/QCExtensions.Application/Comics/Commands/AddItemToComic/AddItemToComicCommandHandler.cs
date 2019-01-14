using MediatR;
using QCExtensions.Application.Extensions.DbContext;
using QCExtensions.Application.Interfaces;
using QCExtensions.Domain.Entities;
using System.Threading;
using System.Threading.Tasks;

namespace QCExtensions.Application.Comics.Commands.AddItemToComic
{
	public class AddItemToComicCommandHandler : IRequestHandler<AddItemToComicCommand, Unit>
	{
		private readonly DomainDbContext _context;
		private readonly IActionLogger _actionLogger;

		public AddItemToComicCommandHandler(
			DomainDbContext context,
			IActionLogger actionLogger
			)
		{
			_context = context;
			_actionLogger = actionLogger;
		}

		public async Task<Unit> Handle(AddItemToComicCommand request, CancellationToken cancellationToken)
		{
			using (var transaction = _context.Database.BeginTransaction())
			{
				var (comic, wasCreated) = await _context.Comics.GetOrCreateAsync(request.ComicId);
				if (wasCreated) await _context.SaveChangesAsync(cancellationToken);

				Item item;
				if (request.ItemId == AddItemToComicCommand.CreateNewItemId)
				{
					item = new Item
					{
						Name = request.NewItemName,
						ShortName = request.NewItemName,
						Type = request.NewItemType
					};

					_context.Items.Add(item);
					await _context.SaveChangesAsync(cancellationToken);
					await _actionLogger.LogAsync(request.Token.Value, $"Created {item.Type} #{item.Id} ({item.Name})", cancellationToken: cancellationToken);
				}
				else
				{
					item = await _context.Items.GetByIdAsync(request.ItemId);
					if (item == null)
					{
						throw new AddItemToComicException("Item does not exist");
					}

					if (await _context.Occurrences.ExistsAsync(comic.Id, item.Id))
					{
						throw new AddItemToComicException("Item is already added to comic");
					}
				}

				_context.Occurrences.Add(new Occurrence { Comic = comic, Item = item });
				await _context.SaveChangesAsync(cancellationToken);
				await _actionLogger.LogAsync(request.Token.Value, $"Added {item.Type} #{item.Id} ({item.Name}) to comic #{comic.Id}", cancellationToken: cancellationToken);

				transaction.Commit();
			}
			return Unit.Value;
		}
	}
}
