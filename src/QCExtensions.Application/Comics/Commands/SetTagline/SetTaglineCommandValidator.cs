using FluentValidation;

namespace QCExtensions.Application.Comics.Commands.SetTagline
{
	public class SetTaglineCommandValidator : AbstractValidator<SetTaglineCommand>
	{
		public SetTaglineCommandValidator()
		{
			RuleFor(x => x.ComicId).GreaterThanOrEqualTo(1);
		}
	}
}
