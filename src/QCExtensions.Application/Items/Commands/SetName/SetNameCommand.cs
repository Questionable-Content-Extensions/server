namespace QCExtensions.Application.Items.Commands.SetName
{
	public class SetNameCommand : RequestWithToken
	{
		public int ItemId { get; set; }
		public string Name { get; set; }
	}
}
