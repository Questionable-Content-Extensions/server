using QCExtensions.Application.Interfaces;

namespace QCExtensions.Application.Items.Commands.AddImage
{

	public class AddImageCommand : RequestWithToken
	{
		public override Permission RequiredPermissions => Permission.CanAddImageToItem;

		public int ItemId { get; set; }
		public uint? CRC32CHash { get; set; }
		public byte[] Image { get; set; }
		public string ImageContentType { get; set; }
	}
}
