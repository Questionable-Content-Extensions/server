using System.Threading;
using System.Threading.Tasks;
using Force.Crc32;
using MediatR;
using QCExtensions.Application.Extensions.DbContext;
using QCExtensions.Application.Interfaces;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Application.Items.Commands.SetName
{
	public class SetNameCommandHandler : IRequestHandler<SetNameCommand, Unit>
	{
		private readonly DomainDbContext _context;
		private readonly IActionLogger _actionLogger;

		public SetNameCommandHandler(
			DomainDbContext context,
			IActionLogger actionLogger
			)
		{
			_context = context;
			_actionLogger = actionLogger;
		}

		public async Task<Unit> Handle(SetNameCommand request, CancellationToken cancellationToken)
		{
			var item = await _context.Items.GetByIdAsync(request.ItemId);
			if (item == null)
			{
				throw new Exceptions.ItemDoesNotExistException();
			}

			var oldValue = item.Name;
			item.Name = request.Name;

			using (var transaction = _context.Database.BeginTransaction())
			{
				_context.Items.Update(item);
				await _context.SaveChangesAsync();

				if (string.IsNullOrEmpty(oldValue))
				{
					await _actionLogger.LogAsync(request.Token.Value, $"Set name of {item.Type} #{item.Id} to \"{request.Name}\"");
				}
				else
				{
					await _actionLogger.LogAsync(request.Token.Value, $"Changed name of {item.Type} #{item.Id} from {oldValue} to \"{request.Name}\"");
				}
				transaction.Commit();
			}

			return Unit.Value;
		}
	}
}
