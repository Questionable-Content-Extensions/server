using FluentValidation;

namespace QCExtensions.Application.Comics.Commands.RemoveItemFromComic
{
	public class RemoveItemFromComicCommandValidator : AbstractValidator<RemoveItemFromComicCommand>
	{
		public RemoveItemFromComicCommandValidator()
		{
			RuleFor(x => x.ComicId).GreaterThanOrEqualTo(1);
			RuleFor(x => x.ItemId).GreaterThanOrEqualTo(1);
		}
	}
}
