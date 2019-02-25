using FluentValidation;
using Force.Crc32;

namespace QCExtensions.Application.Items.Commands.SetShortName
{
	public class SetShortNameCommandValidator : AbstractValidator<SetShortNameCommand>
	{
		public SetShortNameCommandValidator()
		{
			RuleFor(x => x.ItemId).GreaterThanOrEqualTo(1);

			RuleFor(x => x.ShortName).NotEmpty();
		}
	}
}
