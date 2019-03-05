using QCExtensions.Application.Interfaces;
using QCExtensions.Domain.ValueObjects;

namespace QCExtensions.Application.Items.Commands.SetColor
{
	public class SetColorCommand : RequestWithToken
	{
		public override Permission RequiredPermissions => Permission.CanChangeItemData;

		public int ItemId { get; set; }
		public HexRgbColor Color { get; set; }
	}
}
