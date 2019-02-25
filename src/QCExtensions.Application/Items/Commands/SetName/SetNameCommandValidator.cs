using FluentValidation;
using Force.Crc32;

namespace QCExtensions.Application.Items.Commands.SetName
{
	public class SetNameCommandValidator : AbstractValidator<SetNameCommand>
	{
		public SetNameCommandValidator()
		{
			RuleFor(x => x.ItemId).GreaterThanOrEqualTo(1);

			RuleFor(x => x.Name).NotEmpty();
		}
	}
}
