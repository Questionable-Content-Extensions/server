using System.Threading;
using System.Threading.Tasks;
using MediatR;
using QCExtensions.Application.Extensions.DbContext;
using QCExtensions.Application.Interfaces;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Application.Comics.Commands.SetFlag
{
	public class SetFlagCommandHandler : IRequestHandler<SetFlagCommand, Unit>
	{
		private readonly DomainDbContext _context;
		private readonly IActionLogger _actionLogger;

		public SetFlagCommandHandler(
			DomainDbContext context,
			IActionLogger actionLogger
			)
		{
			_context = context;
			_actionLogger = actionLogger;
		}

		public async Task<Unit> Handle(SetFlagCommand request, CancellationToken cancellationToken)
		{
			using (var transaction = _context.Database.BeginTransaction())
			{
				var (comic, wasCreated) = await _context.Comics.GetOrCreateAsync(request.ComicId);
				if (wasCreated) await _context.SaveChangesAsync(cancellationToken);

				string trueValueLogText, falseValueLogText;
				switch (request.Flag)
				{
					case SetFlagCommand.FlagType.IsGuestComic:
						comic.IsGuestComic = request.FlagValue;
						trueValueLogText = "to be a Jeph comic";
						falseValueLogText = "to be a guest comic";
						break;

					case SetFlagCommand.FlagType.IsNonCanon:
						comic.IsNonCanon = request.FlagValue;
						trueValueLogText = "to be non-canon";
						falseValueLogText = "to be canon";
						break;

					case SetFlagCommand.FlagType.HasNoCast:
						comic.HasNoCast = request.FlagValue;
						trueValueLogText = "to have no cast";
						falseValueLogText = "to have cast";
						break;

					case SetFlagCommand.FlagType.HasNoLocation:
						comic.HasNoLocation = request.FlagValue;
						trueValueLogText = "to have no locations";
						falseValueLogText = "to have locations";
						break;

					case SetFlagCommand.FlagType.HasNoStoryline:
						comic.HasNoStoryline = request.FlagValue;
						trueValueLogText = "to have no storylines";
						falseValueLogText = "to have storylines";
						break;

					case SetFlagCommand.FlagType.HasNoTitle:
						comic.HasNoTitle = request.FlagValue;
						trueValueLogText = "to have no title";
						falseValueLogText = "to have a title";
						break;

					case SetFlagCommand.FlagType.HasNoTagline:
						comic.HasNoTagline = request.FlagValue;
						trueValueLogText = "to have no tagline";
						falseValueLogText = "to have a tagline";
						break;

					default:
						throw new SetFlagException($"Unsupported flag: {request.Flag}");
				}
				await _context.SaveChangesAsync(cancellationToken);
				await _actionLogger.LogAsync(request.Token.Value, $"Set comic #{request.ComicId} {(request.FlagValue ? trueValueLogText : falseValueLogText)}", cancellationToken: cancellationToken);

				transaction.Commit();
			}
			return Unit.Value;
		}
	}
}
