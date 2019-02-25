namespace QCExtensions.Application.Items.Commands.AddImage
{

	public class AddImageCommand : RequestWithToken
	{
		public int ItemId { get; set; }
		public uint? CRC32CHash { get; set; }
		public byte[] Image { get; set; }
		public string ImageContentType { get; set; }
	}
}
