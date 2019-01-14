using FluentValidation;

namespace QCExtensions.Application.Comics.Commands.SetTitle
{
	public class SetTitleCommandValidator : AbstractValidator<SetTitleCommand>
	{
		public SetTitleCommandValidator()
		{
			RuleFor(x => x.ComicId).GreaterThanOrEqualTo(1);
		}
	}
}
