using System.Threading;
using System.Threading.Tasks;
using MediatR;
using QCExtensions.Application.Extensions.DbContext;
using QCExtensions.Application.Interfaces;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Application.Comics.Commands.SetPublishDate
{
	public class SetPublishDateCommandHandler : IRequestHandler<SetPublishDateCommand, Unit>
	{
		private readonly DomainDbContext _context;
		private readonly IActionLogger _actionLogger;

		public SetPublishDateCommandHandler(
			DomainDbContext context,
			IActionLogger actionLogger
			)
		{
			_context = context;
			_actionLogger = actionLogger;
		}

		public async Task<Unit> Handle(SetPublishDateCommand request, CancellationToken cancellationToken)
		{
			using (var transaction = _context.Database.BeginTransaction())
			{
				var (comic, wasCreated) = await _context.Comics.GetOrCreateAsync(request.ComicId);
				if (wasCreated) await _context.SaveChangesAsync(cancellationToken);

				var oldPublishDate = comic.PublishDate;
				comic.PublishDate = request.PublishDate;
				comic.IsAccuratePublishDate = request.IsAccuratePublishDate;
				await _context.SaveChangesAsync(cancellationToken);

				if (!oldPublishDate.HasValue)
				{
					await _actionLogger.LogAsync(request.Token.Value, $"Set publish date on comic #{request.ComicId} to \"{request.PublishDate:s}Z\"", cancellationToken: cancellationToken);
				}
				else
				{
					await _actionLogger.LogAsync(request.Token.Value, $"Changed publish date on comic #{request.ComicId} from \"{oldPublishDate:s}Z\" to \"{request.PublishDate:s}Z\"", cancellationToken: cancellationToken);
				}

				transaction.Commit();
			}
			return Unit.Value;
		}
	}
}
