using MediatR;
using QCExtensions.Application.Extensions.DbContext;
using QCExtensions.Application.Interfaces;
using QCExtensions.Domain.Entities;
using System.Threading;
using System.Threading.Tasks;

namespace QCExtensions.Application.Comics.Commands.SetTitle
{
	public class SetTitleCommandHandler : IRequestHandler<SetTitleCommand, Unit>
	{
		private readonly DomainDbContext _context;
		private readonly IActionLogger _actionLogger;

		public SetTitleCommandHandler(
			DomainDbContext context,
			IActionLogger actionLogger
			)
		{
			_context = context;
			_actionLogger = actionLogger;
		}

		public async Task<Unit> Handle(SetTitleCommand request, CancellationToken cancellationToken)
		{
			using (var transaction = _context.Database.BeginTransaction())
			{
				var (comic, wasCreated) = await _context.Comics.GetOrCreateAsync(request.ComicId);
				if (wasCreated) await _context.SaveChangesAsync(cancellationToken);

				var oldTitle = comic.Title;
				comic.Title = request.Title;
				await _context.SaveChangesAsync(cancellationToken);

				if (string.IsNullOrEmpty(oldTitle))
				{
					await _actionLogger.LogAsync(request.Token.Value, $"Set title on comic #{request.ComicId} to \"{request.Title}\"", cancellationToken: cancellationToken);
				}
				else
				{
					await _actionLogger.LogAsync(request.Token.Value, $"Changed title on comic #{request.ComicId} from \"{oldTitle}\" to \"{request.Title}\"", cancellationToken: cancellationToken);
				}

				transaction.Commit();
			}
			return Unit.Value;
		}
	}
}
