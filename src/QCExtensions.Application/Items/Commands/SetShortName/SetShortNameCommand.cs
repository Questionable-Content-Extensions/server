namespace QCExtensions.Application.Items.Commands.SetShortName
{
	public class SetShortNameCommand : RequestWithToken
	{
		public int ItemId { get; set; }
		public string ShortName { get; set; }
	}
}
