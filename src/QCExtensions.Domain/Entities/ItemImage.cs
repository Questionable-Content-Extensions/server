namespace QCExtensions.Domain.Entities
{
	public class ItemImage
	{
		public int Id { get; set; }
		public int ItemId { get; set; }
		public Item Item { get; set; }
		public byte[] Image { get; set; }
		public uint CRC32CHash { get; set; }
	}
}