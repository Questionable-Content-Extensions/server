using QCExtensions.Application.Interfaces;

namespace QCExtensions.Application.Items.Commands.SetName
{
	public class SetNameCommand : RequestWithToken
	{
		public override Permission RequiredPermissions => Permission.CanChangeItemData;

		public int ItemId { get; set; }
		public string Name { get; set; }
	}
}
