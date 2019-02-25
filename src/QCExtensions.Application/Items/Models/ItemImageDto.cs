using System;
using QCExtensions.Application.Interfaces.Mapping;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Application.Items.Models
{
	public class ItemImageDto : IMapFrom<ItemImage>
	{
		public int Id { get; set; }
		public uint CRC32CHash { get; set; }
	}
}
