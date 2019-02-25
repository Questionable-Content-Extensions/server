using FluentValidation;
using Force.Crc32;

namespace QCExtensions.Application.Items.Commands.AddImage
{
	public class AddImageCommandValidator : AbstractValidator<AddImageCommand>
	{
		public AddImageCommandValidator()
		{
			RuleFor(x => x.ItemId).GreaterThanOrEqualTo(1);

			RuleFor(x => x.CRC32CHash)
				.Must((c, v) => Crc32CAlgorithm.Compute(c.Image) == v.Value)
				.Unless(x => !x.CRC32CHash.HasValue)
				.WithMessage("CRC32C value of image does not match with value provided");

			RuleFor(x => x.Image).NotEmpty();

			RuleFor(x => x.ImageContentType)
				.Equal("image/png")
				.WithMessage("Only PNG images are supported");
		}
	}
}
