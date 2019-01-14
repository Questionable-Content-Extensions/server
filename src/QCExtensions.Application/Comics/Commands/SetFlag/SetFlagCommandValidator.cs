using FluentValidation;

namespace QCExtensions.Application.Comics.Commands.SetFlag
{
	public class SetFlagCommandValidator : AbstractValidator<SetFlagCommand>
	{
		public SetFlagCommandValidator()
		{
			RuleFor(x => x.ComicId).GreaterThanOrEqualTo(1);
			RuleFor(x => x.Flag).IsInEnum();
		}
	}
}
