using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;

namespace QCExtensions.Server.Models
{
	public class ItemImage
	{
		[Key]
		public int Id { get; set; }

		public int ItemId { get; set; }

		[Required]
		public Item Item { get; set; }

		[Required]
		public byte[] Image { get; set; }

		[Required]
		public uint CRC32CHash { get; set; }
	}
}