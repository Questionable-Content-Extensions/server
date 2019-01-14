using MediatR;
using QCExtensions.Application.Extensions.DbContext;
using QCExtensions.Application.Interfaces;
using QCExtensions.Domain.Entities;
using System.Threading;
using System.Threading.Tasks;

namespace QCExtensions.Application.Comics.Commands.SetTagline
{
	public class SetTaglineCommandHandler : IRequestHandler<SetTaglineCommand, Unit>
	{
		private readonly DomainDbContext _context;
		private readonly IActionLogger _actionLogger;

		public SetTaglineCommandHandler(
			DomainDbContext context,
			IActionLogger actionLogger
			)
		{
			_context = context;
			_actionLogger = actionLogger;
		}

		public async Task<Unit> Handle(SetTaglineCommand request, CancellationToken cancellationToken)
		{
			using (var transaction = _context.Database.BeginTransaction())
			{
				var (comic, wasCreated) = await _context.Comics.GetOrCreateAsync(request.ComicId);
				if (wasCreated) await _context.SaveChangesAsync(cancellationToken);

				var oldTagline = comic.Tagline;
				comic.Tagline = request.Tagline;
				await _context.SaveChangesAsync(cancellationToken);

				if (string.IsNullOrEmpty(oldTagline))
				{
					await _actionLogger.LogAsync(request.Token.Value, $"Set tagline on comic #{request.ComicId} to \"{request.Tagline}\"", cancellationToken: cancellationToken);
				}
				else
				{
					await _actionLogger.LogAsync(request.Token.Value, $"Changed tagline on comic #{request.ComicId} from \"{oldTagline}\" to \"{request.Tagline}\"", cancellationToken: cancellationToken);
				}

				transaction.Commit();
			}
			return Unit.Value;
		}
	}
}
