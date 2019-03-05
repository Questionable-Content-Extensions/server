using QCExtensions.Application.Interfaces;

namespace QCExtensions.Application.Items.Commands.SetShortName
{
	public class SetShortNameCommand : RequestWithToken
	{
		public override Permission RequiredPermissions => Permission.CanChangeItemData;

		public int ItemId { get; set; }
		public string ShortName { get; set; }
	}
}
