using QCExtensions.Domain.ValueObjects;

namespace QCExtensions.Application.Items.Commands.SetColor
{
	public class SetColorCommand : RequestWithToken
	{
		public int ItemId { get; set; }
		public HexRgbColor Color { get; set; }
	}
}
