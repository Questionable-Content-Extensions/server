using FluentValidation;
using Force.Crc32;

namespace QCExtensions.Application.Items.Commands.SetColor
{
	public class SetColorCommandValidator : AbstractValidator<SetColorCommand>
	{
		public SetColorCommandValidator()
		{
			RuleFor(x => x.ItemId).GreaterThanOrEqualTo(1);

			RuleFor(x => x.Color).NotEmpty();
		}
	}
}
